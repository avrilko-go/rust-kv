use std::marker::PhantomData;
use bytes::BytesMut;

pub struct ProstStream<S, In, Out> {
    stream: S,
    wbuf: BytesMut,
    written: usize,
    rbuf: BytesMut,
    _in: PhantomData<In>,
    _out: PhantomData<Out>,
}

impl<S, In, Out> ProstStream<S, In, Out> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            written: 0,
            wbuf: BytesMut::new(),
            rbuf: BytesMut::new(),
            _in: PhantomData::default(),
            _out: PhantomData::default(),
        }
    }
}