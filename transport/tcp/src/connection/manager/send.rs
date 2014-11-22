
pub struct SendMgr{
  // State Variables
  // TODO: sizes of all these
  send_UNA : uint,   // Oldest unacknowledged sequence number
  send_NXT : uint,   // Next Sequence Number to be Sent
  send_WND : uint,   // Send Window Size
  //send_UP  : uint, // send urgent pointer
  send_WL1 : uint,   // Seq number for last window update
  send_WL2 : uint,   // Ack number for last window update
  send_ISN : uint,   // Send Initial Sequence Number
}

//TODO: all the things
impl SendMgr {
  pub fn new() -> SendMgr {
    SendMgr{
      send_UNA : 0u,
      send_NXT : 0u,
      send_WND : 0u,
      send_WL1 : 0u,
      send_WL2 : 0u,
      send_ISN : 0u, //TODO: randomly generate
    }
  } 

  pub fn send(&self, buf: &[u8], start: u32, n: uint) -> uint {
    //TODO: 
    return 0u;
  }
}
