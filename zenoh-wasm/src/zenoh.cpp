//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

#include "zenoh-pico.h"
#include "zenoh-pico/api/macros.h"
#include "zenoh-pico/api/types.h"
// #include "zenoh-pico/api.h"
#include "zenoh-pico/system/platform.h"
#include <chrono>
#include <cstdlib>
#include <emscripten/bind.h>
#include <emscripten/emscripten.h>
#include <emscripten/val.h>
#include <iostream>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <thread>
#include <unistd.h>

extern void remove_js_callback(void *);

extern "C" {

EMSCRIPTEN_KEEPALIVE
void test_sleep(int ms) { sleep(ms); }

EMSCRIPTEN_KEEPALIVE
int zw_version() { return Z_PROTO_VERSION; }

EMSCRIPTEN_KEEPALIVE
void *zw_default_config(const char *locator) {
  if (locator == NULL) {
    return NULL;
  }

  z_owned_config_t *config =
      (z_owned_config_t *)z_malloc(sizeof(z_owned_config_t));
  *config = z_config_default();
  zp_config_insert(z_loan(*config), Z_CONFIG_CONNECT_KEY,
                   z_string_make(locator));
  return (void *)config;
}

EMSCRIPTEN_KEEPALIVE
void *zw_open_session(z_owned_config_t *config) {
  z_owned_session_t *session =
      (z_owned_session_t *)z_malloc(sizeof(z_owned_session_t));
  *session = z_open(z_move(*config));
  if (!z_check(*session)) {
    printf("Unable to open session!\n");
    z_free(session);
    return NULL;
  }
  return session;
}

EMSCRIPTEN_KEEPALIVE
void *zw_session_close(z_owned_config_t *config) {
  z_owned_session_t *session =
      (z_owned_session_t *)z_malloc(sizeof(z_owned_session_t));
  // *session = z_open(z_move(*config));
  // if (!z_check(*session))
  // {
  //   printf("Unable to open session!\n");
  //   z_free(session);
  //   return NULL;
  // }
  // return session;
}

EMSCRIPTEN_KEEPALIVE
int zw_start_tasks(z_owned_session_t *s) {
  if (zp_start_read_task(z_loan(*s), NULL) < 0 ||
      zp_start_lease_task(z_loan(*s), NULL) < 0) {
    printf("Unable to start read and lease tasks");
    return -1;
  }
  return 0;
}

EMSCRIPTEN_KEEPALIVE
z_owned_keyexpr_t *zw_make_ke(const char *keyexpr) {
  z_owned_keyexpr_t *ke = NULL;
  z_owned_keyexpr_t oke = z_keyexpr_new(keyexpr);
  if (z_check(oke)) {
    ke = (z_owned_keyexpr_t *)z_malloc(sizeof(z_owned_keyexpr_t));
    _z_keyexpr_set_owns_suffix(oke._value, true);
    *ke = oke;
  }
  return ke;
}

EMSCRIPTEN_KEEPALIVE
void *zw_declare_ke(z_owned_session_t *s, const char *keyexpr) {

  z_owned_keyexpr_t *ke =
      (z_owned_keyexpr_t *)z_malloc(sizeof(z_owned_keyexpr_t));
  z_keyexpr_t key = z_keyexpr(keyexpr);
  *ke = z_declare_keyexpr(z_loan(*s), key);
  if (!z_check(*ke)) {
    printf("Unable to declare key expression!\n");
    exit(-1);
  }
  return ke;
}

EMSCRIPTEN_KEEPALIVE
void *zw_subscriber(const z_owned_session_t *s,
                    const z_owned_keyexpr_t *keyexpr) {}

EMSCRIPTEN_KEEPALIVE
void zw_delete_ke(z_owned_keyexpr_t *keyexpr) { return z_drop(keyexpr); }

// TODO Complete
// EMSCRIPTEN_KEEPALIVE
// int zw_get(z_owned_session_t *s, // TODO: Do I need an owned session T ?
//            z_owned_keyexpr_t *ke,
//            // z_session_t *s,
//            //  z_keyexpr_t *ke,
//            const char *parameters,
//            int js_callback)
// {
//   z_get_options_t options = z_get_options_default();

//   z_owned_closure_sample_t callback =
//       z_closure(wrapping_sub_callback, remove_js_callback, (void
//       *)js_callback);

//   int8_t get = z_get(z_loan(*s), z_loan(*ke), parameters, z_move(callback),
//   &options);

//   return get;
// }

EMSCRIPTEN_KEEPALIVE
int zw_put(z_owned_session_t *s, z_owned_keyexpr_t *ke, char *value, int len) {
  z_put_options_t options = z_put_options_default();
  options.encoding = z_encoding(Z_ENCODING_PREFIX_TEXT_PLAIN, NULL);
  // TODO FIX
  // return z_put(z_loan(*s), z_loan(*ke), value, len, &options);
  return 10;
}

EMSCRIPTEN_KEEPALIVE
void spin(z_owned_session_t *s) {
  zp_read(z_loan(*s), NULL);
  zp_send_keep_alive(z_loan(*s), NULL);
  // zp_send_join(z_loan(*s), NULL);
}

EMSCRIPTEN_KEEPALIVE
void close_session(z_owned_session_t *s) { z_close(z_move(*s)); }

// TODO
// TODO
// TODO
// TODO
// TODO
// EMSCRIPTEN_KEEPALIVE
// void *zw_sub(z_owned_session_t *s, z_owned_keyexpr_t *ke, int js_callback)
// {
//   z_owned_subscriber_t *sub =
//       (z_owned_subscriber_t *)z_malloc(sizeof(z_owned_subscriber_t));
//   // TODO
//   // z_owned_closure_sample_t callback =
//   //     z_closure(wrapping_sub_callback, remove_js_callback, (void
//   *)js_callback); *sub = z_declare_subscriber(z_loan(*s), z_loan(*ke),
//   z_move(callback), NULL); if (!z_check(*sub))
//   {
//     printf("Unable to declare subscriber.\n");
//     exit(-1);
//   }
//   return sub;
// }
// TODO
// TODO
// TODO
// TODO
// TODO

EMSCRIPTEN_KEEPALIVE
void z_wasm_free(void *ptr) { z_free(ptr); }

// ███    ██ ███████  ██████
// ████   ██ ██      ██    ██
// ██ ██  ██ █████   ██    ██
// ██  ██ ██ ██      ██    ██
// ██   ████ ███████  ██████

// Horrible
// int zw_put(z_owned_session_t *s, 
//            z_owned_keyexpr_t *ke, 
//            char *value, 
//            int len) {}
int neo_zw_put(emscripten::val session,
               emscripten::val key_expr, std::string value) {

  printf("------ neo_zw_put ------\n");
  for (unsigned char item : value) {
    std::cout << item << std::endl;
  }

  int len = value.length();

  z_put_options_t options = z_put_options_default();
  options.encoding = z_encoding(Z_ENCODING_PREFIX_TEXT_PLAIN, NULL);

    // z_owned_session_t *s, 
    //            z_owned_keyexpr_t *ke,
    // std::cout << session.typeof() << std::endl;
    // std::cout << key_expr.typeof() << std::endl;
    std::cout << value << std::endl;

  // TODO FIX
  // return z_put(z_loan(session), z_loan(key_expr), value, len, &options);
  return 10 ;
}

// ██████  ███████ ██    ██
// ██   ██ ██      ██    ██
// ██   ██ █████   ██    ██
// ██   ██ ██       ██  ██
// ██████  ███████   ████

// C++ Way of Calling Callbacks
// cb : Async Function from JS
// cb : is a js object, ripe for any and all JS fuckery
int callback_test_async(emscripten::val cb) {
  printf("------ callback_test_async ------\n");

  int ret = cb(5).await().as<int>();

  return ret;
}

int callback_test(emscripten::val cb) {
  printf("------ callback_test ------\n");

  int ret = cb(5).as<int>();

  printf("   ret val: %d \n", ret);

  return ret;
}

int pass_arr_cpp(std::string js_arr) {

  printf("------ pass_arr_cpp ------\n");
  for (unsigned char item : js_arr) {
    std::cout << item << std::endl;
  }
  return 10;
}

// Macro to Expose Functions
EMSCRIPTEN_BINDINGS(my_module) {
  emscripten::function("callback_test", &callback_test);
  emscripten::function("callback_test_async", &callback_test_async);
  emscripten::function("pass_arr_cpp", &pass_arr_cpp);
  emscripten::function("neo_zw_put", &neo_zw_put);
}
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
}