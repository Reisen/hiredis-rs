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

pub enum ReplyCode {
    String = 1,
    Array,
    Integer,
    Nil,
    Status,
    Error,
    Unknown
}

impl Reply {
    unsafe fn new(reply: *const api::Reply) -> Reply {
        Reply {
            reply: reply
        }
    }

    pub fn typename(&self) -> ReplyCode {
        unsafe {
            match (*self.reply)._type {
                1 => String,
                2 => Array,
                3 => Integer,
                4 => Nil,
                5 => Status,
                6 => Error,
                _ => Unknown
            }
        }
    }
}

impl Drop for Reply {
    fn drop(&mut self) {
        unsafe {
            api::freeReplyObject(transmute(self.reply))
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

    pub fn receive(&self) -> Option<Reply> {
        unsafe {
            let reply: Reply = std::mem::uninitialized();
            let result       = api::redisGetReply(self.context, transmute(&reply));

            Some(reply)
        }
    }
}

impl Drop for Redis {
    fn drop(&mut self) {
        unsafe {
            api::redisFree(self.context)
        }
    }
}
