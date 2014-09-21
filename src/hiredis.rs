#![crate_name = "hiredis"]
#![crate_type = "lib"]
#![feature(globs)]
#![feature(tuple_indexing)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate libc;

pub mod api;


pub struct Reply {
    head:  bool,
    reply: *mut api::Reply
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
    /**
     * Creates a new Reply wrapper.
     *
     * This wraps an already existing Reply returned from hiredis, it will
     * automatically free the data when the reply goes out of scope. Note that
     * this method cannot be invoked outside of the crate.
     */
    fn new(reply: *mut api::Reply) -> Reply {
        Reply {
            head:  true,
            reply: reply
        }
    }

    pub fn empty() -> Reply {
        Reply {
            head: true,
            reply: 0 as *mut api::Reply
        }
    }

    /**
     * Redis replies with the type of data being sent, this method allows
     * peaking into it.
     *
     * TODO: Create a nicer wrapper enum that encodes the data directly with
     * Vectors and so on.
     */
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

    pub fn string<'r>(&'r self) -> Option<&'r [u8]> {
        match self.typename() {
            String => {
                unsafe {
                    let data = std::c_vec::CVec::new(
                        (*self.reply)._str as *mut u8,
                        (*self.reply).len as uint
                    ).as_mut_slice() as *mut [u8];

                    Some(&*data)
                }
            }

            _ => {
                None
            }
        }
    }

    pub fn array<'r>(&'r self) -> Vec<Reply> {
        match self.typename() {
            Array => {
                unsafe {
                    /* There is some serious lifetime casting bullshit going on
                     * here, sorry to anyone reading this. */
                    let data = std::c_vec::CVec::new(
                        (*self.reply).element as *mut &'r mut api::Reply,
                        (*self.reply).elements as uint
                    ).as_mut_slice() as *mut [&'r mut api::Reply];

                    let mut result = Vec::new();

                    for v in (&mut *data).iter_mut() {
                        result.push(Reply {
                            head:  false,
                            reply: *v
                        });
                    }

                    result
                }
            }

            _ => {
                Vec::new()
            }
        }
    }
}

impl Drop for Reply {
    fn drop(&mut self) {
        unsafe {
            /* When Redis returns a Reply object, we wrap it in our own Reply
             * object. However, if redis returns an array, the Reply object
             * contains an array of sub Reply objects. These get wrapped too,
             * but should not be freed, as freeing the head object frees the
             * children too. This check prevents us accidentally freeing replies
             * that shouldn't be freed.
             */
            if self.head &&
               self.reply != 0 as *mut api::Reply {
                api::freeReplyObject(self.reply as *const libc::c_void);
            }
        }
    }
}

pub struct Redis {
    context: *const api::Context
}

impl Redis {
    /**
     * Creates a new connection to a Redis instance.
     *
     * TODO: Deal with the other redisConnect* variants, not yet sure how to
     * design this API.
     */
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

    /**
     * Sends a single command to be executed, and returns the reply immediately.
     * If further replies are expected the `receive` function exists to expect
     * them. This function doesn't handle complex spaces in messages, and so the
     * `execf` function exists and uses hiredis's formatting instead.
     *
     * TODO: Move to IoResult using context.err
     */
    pub fn command(&self, command: &str) -> Option<Reply> {
        command.with_c_str(|v| {
            unsafe {
                /* This is the library call, returns (void*)0 on failure. */
                let result = api::redisCommand(self.context, v);

                /* Fail if the command errored for some reason. */
                if result == 0 as *const ::libc::c_void {
                    return None;
                }

                /* Otherwise cast the pointer to the actual struct type being
                 * returned and store it to be returned. */
                Some(Reply::new(result as *mut api::Reply))
            }
        })
    }

    /**
     * Appends a command to the output buffer, allowing for pipelining.
     *
     * TODO: This needs to return a status code.
     */
    pub fn append_command(&self, command: &str) {
        command.with_c_str(|v| {
            unsafe {
                api::redisAppendCommand(self.context, v);
            }
        });
    }

    /**
     * Attempts to read a message from the input buffer. Blocking in this
     * context depends on how the connection was created.
     */
    pub fn receive(&self, reply: &mut Reply) {
        unsafe {
            /* If there's already a reply allocated, clean it out early so it
             * can be re-used. */
            if reply.reply != 0 as *mut api::Reply {
                api::freeReplyObject(reply.reply as *const libc::c_void);
                reply.reply = 0 as *mut api::Reply;
            }

            api::redisGetReply(
                self.context,
                (&mut reply.reply) as *mut _ as *mut *mut libc::c_void
            );
        }
    }
}

impl Drop for Redis {
    /**
     * When a Redis instance goes out of scope, the connection needs to be
     * closed. Redis has a function for this that both closes the connection and
     * frees any resources used.
     */
    fn drop(&mut self) {
        unsafe {
            api::redisFree(self.context)
        }
    }
}
