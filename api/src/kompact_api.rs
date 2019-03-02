extern crate bytes;
extern crate kompact;

use kompact::prelude::BufMut;
use kompact::*;
pub use crate::messages::messages::MetricReport;
use protobuf::Message;

impl Serialisable for MetricReport {
    fn serid(&self) -> u64 {
        serialisation_ids::PBUF
    }
    fn size_hint(&self) -> Option<usize> {
        if let Ok(bytes) = self.write_to_bytes() {
            Some(bytes.len())
        } else {
            None
        }
    }
    fn serialise(&self, buf: &mut BufMut) -> Result<(), SerError> {
        let bytes = self.write_to_bytes()
            .map_err(|err| SerError::InvalidData(err.to_string()))?;
        buf.put_slice(&bytes);
        Ok(())
    }
    fn local(self: Box<Self>) -> Result<Box<Any + Send>, Box<Serialisable>> {
        Ok(self)
    }
}
