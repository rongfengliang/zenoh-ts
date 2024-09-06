// Remote API interface
import {
  RemoteRecvErr as GetChannelClose,
  RemoteSession,
} from "./remote_api/session";
import { ReplyWS } from "./remote_api/interface/ReplyWS";
import { RemotePublisher, RemoteSubscriber } from "./remote_api/pubsub";
import { SampleWS } from "./remote_api/interface/SampleWS";
import { RemoteQueryable } from "./remote_api/query";
import { QueryWS } from "./remote_api/interface/QueryWS";
// API interface
import { IntoKeyExpr, KeyExpr } from "./key_expr";
import { IntoZBytes, ZBytes } from "./z_bytes";
import {
  IntoSelector,
  Parameters,
  Query,
  Queryable,
  QueryWS_to_Query,
  Reply,
  Selector,
} from "./query";
import { SimpleChannel } from "channel-ts";
import { FifoChannel, Handler, Publisher, Subscriber } from "./pubsub";
import {
  priority_to_int,
  congestion_control_to_int,
  CongestionControl,
  Priority,
  Sample,
  Sample_from_SampleWS,
} from "./sample";
import { State } from "channel-ts/lib/channel";
import { Config } from "./config";
import { Encoding } from "./encoding";
import { QueryReplyWS } from "./remote_api/interface/QueryReplyWS";
import { Error } from "./remote_api/session";

export { Error };
export type Option<T> = T | null;

function executeAsync(func: any) {
  setTimeout(func, 0);
}

// ███████ ███████ ███████ ███████ ██  ██████  ███    ██
// ██      ██      ██      ██      ██ ██    ██ ████   ██
// ███████ █████   ███████ ███████ ██ ██    ██ ██ ██  ██
//      ██ ██           ██      ██ ██ ██    ██ ██  ██ ██
// ███████ ███████ ███████ ███████ ██  ██████  ██   ████

/**
 * Zenoh Session
 */

export class Session {
  // WebSocket Backend
  private remote_session: RemoteSession;

  private constructor(remote_session: RemoteSession) {
    this.remote_session = remote_session;
  }

  /**
   * Creates a new Session instance
   *
   * @remarks
   *  Opens A Zenoh Session
   *
   * @param config - Config for session
   * @returns Typescript instance of a Session
   *
   */

  static async open(config: Promise<Config> | Config): Promise<Session> {
    const cfg = await config;
    let remote_session = await RemoteSession.new(cfg.locator);
    return new Session(remote_session);
  }

  /**
   * Closes a session, cleaning up the resource in Zenoh
   *
   * @returns Nothing
   */
  async close() {
    this.remote_session.close();
  }

  /**
   * Puts a value on the session, on a specific key expression KeyExpr
   *
   * @param keyexpr - something that implements intoKeyExpr
   * @param value - something that implements intoValue
   *
   * @returns void
   */
  async put(
    into_key_expr: IntoKeyExpr,
    into_zbytes: IntoZBytes,
  ): Promise<void> {
    let key_expr = KeyExpr.new(into_key_expr);
    let z_bytes = ZBytes.new(into_zbytes);

    this.remote_session.put(key_expr.toString(), Array.from(z_bytes.payload()));
  }

  async delete(into_key_expr: IntoKeyExpr): Promise<void> {
    let key_expr = KeyExpr.new(into_key_expr);

    this.remote_session.delete(key_expr.toString());
  }

  /**
   * Issues a get query on a Zenoh session
   *
   * @param into_selector - representing a KeyExpr and Parameters
   *
   * @returns Receiver
   */

  async get(
    into_selector: IntoSelector,
    callback?: (reply: Reply) => Promise<void>,
  ): Promise<Receiver | void> {

    let selector: Selector;
    let key_expr: KeyExpr;

    if (typeof into_selector === "string" || into_selector instanceof String) {
      let split_string = into_selector.split("?")
      if (split_string.length == 1) {
        key_expr = KeyExpr.new(into_selector);
        selector = Selector.new(key_expr);
      } else if (split_string.length == 2) {
        key_expr = KeyExpr.new(split_string[0]);
        let parameters: Parameters = Parameters.new(split_string[1]);
        selector = Selector.new(key_expr, parameters);
      } else { //Error in Selector
        throw "Error: Invalid Selector, expected format <KeyExpr>?<Parameters>";
      }
    } else {
      selector = Selector.new(into_selector);
    }

    let chan: SimpleChannel<ReplyWS> = await this.remote_session.get(
      selector.key_expr().toString(),
      selector.parameters().toString(),
    );

    let receiver = Receiver.new(chan);

    if (callback != undefined) {
      executeAsync(async () => {
        for await (const message of chan) {
          // This horribleness comes from SimpleChannel sending a 0 when the channel is closed
          if (message != undefined && (message as unknown as number) != 0) {
            let reply = Reply.new(message);
            callback(reply);
          } else {
            break
          }
        }
      });
      return undefined;
    } else {
      return receiver;
    }
  }

  /**
   * Declares a new subscriber
   *
   * @remarks
   *  If a Subscriber is created with a callback, it cannot be simultaneously polled for new values
   * 
   * @param keyexpr - string of key_expression
   * @param callback function - Function to be called for all samples
   *
   * @returns Subscriber
   */
  async declare_subscriber(
    into_key_expr: IntoKeyExpr,
    handler: Handler = new FifoChannel(256), // This is to match the API_DATA_RECEPTION_CHANNEL_SIZE of zenoh internally
    callback?: (sample: Sample) => Promise<void>,
  ): Promise<Subscriber> {
    let key_expr = KeyExpr.new(into_key_expr);
    let remote_subscriber: RemoteSubscriber;
    let callback_subscriber = false;
    if (callback != undefined) {
      callback_subscriber = true;
      const callback_conversion = async function (
        sample_ws: SampleWS,
      ): Promise<void> {
        let sample: Sample = Sample_from_SampleWS(sample_ws);
        callback(sample);
      };
      remote_subscriber = await this.remote_session.declare_subscriber(
        key_expr.toString(),
        handler,
        callback_conversion,
      );
    } else {
      remote_subscriber = await this.remote_session.declare_subscriber(
        key_expr.toString(),
        handler,
      );
    }

    let subscriber = await Subscriber.new(
      remote_subscriber,
      callback_subscriber,
      
    );
    return subscriber;
  }

  /**
  * Declares a new Queryable
  *
  * @remarks
  *  If a Queryable is created with a callback, it cannot be simultaneously polled for new Query's
  * 
  * @param keyexpr - string of key_expression
  * @param complete - boolean representing Queryable completeness
  * @param callback function - Function to be called for all samples
  *
  * @returns Queryable
  */
  async declare_queryable(
    into_key_expr: IntoKeyExpr,
    complete: boolean,
    callback?: (query: Query) => Promise<void>,
  ): Promise<Queryable> {
    let key_expr = KeyExpr.new(into_key_expr);
    let remote_queryable: RemoteQueryable;
    let reply_tx: SimpleChannel<QueryReplyWS> =
      new SimpleChannel<QueryReplyWS>();

    if (callback != undefined) {
      const callback_conversion = async function (
        query_ws: QueryWS,
      ): Promise<void> {
        let query: Query = QueryWS_to_Query(query_ws, reply_tx);

        callback(query);
      };
      remote_queryable = await this.remote_session.declare_queryable(
        key_expr.toString(),
        complete,
        reply_tx,
        callback_conversion,
      );
    } else {
      remote_queryable = await this.remote_session.declare_queryable(
        key_expr.toString(),
        complete,
        reply_tx,
      );
    }

    let queryable = await Queryable.new(remote_queryable);
    return queryable;
  }

  /**
  * Declares a new Publisher
  *
  * @remarks
  *  If a Queryable is created with a callback, it cannot be simultaneously polled for new Query's
  * 
  * @param keyexpr - string of key_expression
  * @param encoding - Optional, Type of Encoding data to be sent over
  * @param congestion_control - Optional, Type of Congestion control to be used (BLOCK / DROP)
  * @param priority - Optional, The Priority of zenoh messages
  *
  * @returns Publisher
  */
  async declare_publisher(
    keyexpr: IntoKeyExpr,
    encoding?: Encoding,
    congestion_control?: CongestionControl,
    priority?: Priority,
    express?: boolean,
  ): Promise<Publisher> {
    let key_expr: KeyExpr = KeyExpr.new(keyexpr);

    let _congestion_ctrl = 0; // Default CongestionControl.DROP
    if (congestion_control != null) {
      _congestion_ctrl = congestion_control_to_int(congestion_control);
    } else {
      congestion_control = CongestionControl.DROP;
    }

    let _priority = 5; // Default Priority.DATA
    if (priority != null) {
      _priority = priority_to_int(priority);
    } else {
      priority = Priority.DATA;
    }

    let _express = false;
    if (express != null) {
      _express = express;
    }

    let _encoding = Encoding.default();
    if (encoding != null) {
      _encoding = encoding;
    }

    let remote_publisher: RemotePublisher =
      await this.remote_session.declare_publisher(
        key_expr.toString(),
        _encoding.toString(),
        _congestion_ctrl,
        _priority,
        _express,
      );

    var publisher: Publisher = await Publisher.new(
      key_expr,
      remote_publisher,
      congestion_control,
      priority,
    );
    return publisher;
  }
}

function isGetChannelClose(msg: any): msg is GetChannelClose {
  return msg === GetChannelClose.Disconnected;
}

// Type guard to check if channel_msg is of type ReplyWS
function isReplyWS(msg: any): msg is ReplyWS {
  return (
    typeof msg === "object" &&
    msg !== null &&
    "query_uuid" in msg &&
    "result" in msg
  );
}

export enum RecvErr {
  Disconnected,
  MalformedReply,
}


/**
 * Receiver returned from `get` call on a session
 */

export class Receiver {
  private receiver: SimpleChannel<ReplyWS | RecvErr>;

  private constructor(receiver: SimpleChannel<ReplyWS | RecvErr>) {
    this.receiver = receiver;
  }

  /**
   *  Receives next Reply message from Zenoh `get`
   * 
   * @returns Reply
   */
  async receive(): Promise<Reply | RecvErr> {
    if (this.receiver.state == State.close) {
      return RecvErr.Disconnected;
    } else {
      let channel_msg: ReplyWS | RecvErr = await this.receiver.receive();

      if (isGetChannelClose(channel_msg)) {
        return RecvErr.Disconnected;
      } else if (isReplyWS(channel_msg)) {
        // Handle the ReplyWS case
        let opt_reply = Reply.new(channel_msg);
        if (opt_reply == undefined) {
          return RecvErr.MalformedReply;
        } else {
          return opt_reply;
        }
      }
      return RecvErr.MalformedReply;
    }
  }
  static new(reply_tx: SimpleChannel<ReplyWS>) {
    return new Receiver(reply_tx);
  }
}

/**
 *  Function to open a Zenoh session
 */
export function open(config: Config): Promise<Session> {
  return Session.open(config);
}
