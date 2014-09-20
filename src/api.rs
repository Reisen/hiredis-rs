use libc::{
    c_char,
    c_int,
    c_longlong,
    c_void,
    size_t
};

pub enum ReplyCode {
    String,
    Array,
    Integer,
    Nil,
    Status,
    Error
}

#[repr(C)]
pub struct Reply {
    _type:    c_int,
    integer:  c_longlong,
    len:      c_int,
    _str:     *const c_char,
    elements: size_t,
    element:  *const (*const Reply)
}

#[repr(C)]
pub struct ReadTask {
    _type:    c_int,
    elements: c_int,
    idx:      c_int,
    obj:      *const c_void,
    parent:   *const ReadTask,
    privdata: *const c_void
}

/* TODO: This may be wrong. So far just doing a straight translation from hiredis.h, come back to
 * this in the future and test/fix. */
#[repr(C)]
pub struct ReplyObjectFunctions {
    createString:  extern fn(*const ReadTask, *const c_char, size_t) -> *const c_void,
    createArray:   extern fn(*const ReadTask, c_int) -> *const c_void,
    createInteger: extern fn(*const ReadTask, c_longlong) -> *const c_void,
    createNil:     extern fn(*const ReadTask) -> *const c_void,
    freeObject:    extern fn(*const c_void)
}

#[repr(C)]
pub struct Reader {
    err:      c_int,
    errstr:   [c_char, ..128],
    buf:      *const c_char,
    pos:      size_t,
    len:      size_t,
    maxbuf:   size_t,
    rstack:   [ReadTask, ..9],
    ridx:     c_int,
    reply:    *const c_void,
    _fn:      *const ReplyObjectFunctions,
    privData: *const c_void
}

#[repr(C)]
pub struct Context {
    err:    c_int,
    errstr: [c_char, ..128],
    fd:     c_int,
    flags:  c_int,
    obuf:   *const c_char,
    reader: *const Reader
}

#[link(name = "hiredis")]
extern {
    /* Public API for the Protocol Parser. */
    pub fn redisReaderCreate() -> *const Reader;
    pub fn redisReaderFree(r: *const Reader) -> c_int;
    pub fn redisReaderFeed(r: *const Reader, buf: *const c_char, len: size_t) -> c_int;
    pub fn redisReaderGerReply(r: *const Reader, reply: *const (*const c_void)) -> c_int;
    pub fn freeReplyObject(reply: *const c_void);

    /* TODO: This is wrong, last argument needs to be va_list. Don't know how to do it yet. */
    pub fn redisvFormatCommand(target: *const (*const c_char), format: *const c_char, ap: *const c_void) -> c_int;
    pub fn redisFormatCommand(target: *const (*const c_char), format: *const c_char, ...) -> c_int;
    pub fn redisFormatCommandArgv(target: *const (*const c_char), argc: c_int, argv: *const (*const c_char), argvlen: *const size_t) -> c_int;

    /* TODO: Add redisConnectWithTimeout and redisConnectUnixWithTimeout. */
    pub fn redisConnect(ip: *const c_char, port: c_int) -> *const Context;
    pub fn redisConnectNonBlock(ip: *const c_char, port: c_int) -> *const Context;
    pub fn redisConnectBindNonBlock(ip: *const c_char, port: c_int, source_addr: *const c_char) -> *const Context;
    pub fn redisConnectUnix(path: *const c_char) -> *const Context;
    pub fn redisConnectUnixNonBlock(path: *const c_char) -> *const Context;
    pub fn redisConnectFd(fd: c_int) -> *const Context;

    /* TODO: Add redisSetTimeout */
    pub fn redisEnableKeepAlive(c: *const Context) -> c_int;
    pub fn redisFree(c: *const Context);
    pub fn redisFreeKeepFd(c: *const Context) -> c_int;
    pub fn redisBufferRead(c: *const Context) -> c_int;
    pub fn redisBufferWrite(c: *const Context) -> c_int;

    pub fn redisGetReply(c: *const Context, reply: *const (*const c_void)) -> c_int;
    pub fn redisGetReplyFromReader(c: *const Context, reply: *const (*const c_void)) -> c_int;
    pub fn redisAppendFormattedCommand(c: *const Context, cmd: *const c_char, len: size_t) -> c_int;

    /* TODO: va_list again, needs fixing. */
    pub fn redisvAppendCommand(c: *const Context, format: *const c_char, ap: *const c_void) -> c_int;
    pub fn redisAppendCommand(c: *const Context, format: *const c_char, ...) -> c_int;
    pub fn redisAppendCommandArgv(c: *const Context, argc: c_int, argv: *const (*const c_char), argvlen: *const size_t) -> c_int;

    /* TODO: va_list, again. */
    pub fn redisvCommand(c: *const Context, format: *const c_char, ap: *const c_void) -> *const c_void;
    pub fn redisCommand(c: *const Context, format: *const c_char, ...) -> *const c_void;
    pub fn redisCommandArgv(c: *const Context, argc: c_int, argv: *const (*const c_char), argvlen: *const size_t) -> *const c_void;
}
