#![crate_name = "hiredis"]
#![crate_type = "lib"]
#![feature(globs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate libc;

use std::mem::transmute;

pub mod api;


pub struct Reply {
    reply: *const api::Reply
}

impl Reply {
    unsafe fn new(reply: *const api::Reply) -> Reply {
        Reply {
            reply: reply
        }
    }
}

pub struct Redis {
    context: *const api::Context
}

impl Redis {
    pub fn new(ip: &str, port: i32) -> Redis {
        unsafe {
            Redis {
                context: api::redisConnect(
                    ip.to_c_str().as_ptr(),
                    port
                )
            }
        }
    }

    /* TODO: Move to IoResult using context.err */
    pub fn exec(&self, command: &str) -> Option<Reply> {
        command.with_c_str(|v| {
            unsafe {
                let result = api::redisCommand(self.context, v);

                /* Fail if the command errored for some reason. */
                if result == 0 as *const ::libc::c_void {
                    return None;
                }

                /* Otherwise transmute the void pointer memory into a pointer
                 * to a reply structure and return it. */
                Some(Reply::new(transmute(result)))
            }
        })
    }
}

impl Drop for Redis {
    fn drop(&mut self) {
        unsafe {
            api::redisFree(self.context)
        }
    }
}
