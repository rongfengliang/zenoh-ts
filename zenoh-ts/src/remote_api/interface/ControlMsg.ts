// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { HandlerChannel } from "./HandlerChannel";
import type { OwnedKeyExprWrapper } from "./OwnedKeyExprWrapper";

export type ControlMsg = "OpenSession" | "CloseSession" | { "Session": string } | { "Get": { key_expr: OwnedKeyExprWrapper, parameters: string | null, handler: HandlerChannel, id: string, consolidation: number | undefined, congestion_control: number | undefined, priority: number | undefined, express: boolean | undefined, encoding: string | undefined, payload: number[] | undefined, attachment: number[] | undefined, } } | { "GetFinished": { id: string, } } | { "Put": { key_expr: OwnedKeyExprWrapper, payload: Array<number>, encoding: string | undefined, congestion_control: number | undefined, priority: number | undefined, express: boolean | undefined, attachment: number[] | undefined, } } | { "Delete": { key_expr: OwnedKeyExprWrapper, congestion_control: number | undefined, priority: number | undefined, express: boolean | undefined, attachment: number[] | undefined, } } | { "DeclareSubscriber": { key_expr: OwnedKeyExprWrapper, handler: HandlerChannel, id: string, } } | { "Subscriber": string } | { "UndeclareSubscriber": string } | { "DeclarePublisher": { key_expr: OwnedKeyExprWrapper, encoding: string | undefined, congestion_control: number | undefined, priority: number | undefined, reliability: number | undefined, express: boolean | undefined, id: string, } } | { "UndeclarePublisher": string } | { "DeclareQueryable": { key_expr: OwnedKeyExprWrapper, id: string, complete: boolean, } } | { "UndeclareQueryable": string };
