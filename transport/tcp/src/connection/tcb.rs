use ringbuf::RingBuf;

const TCP_BUF_SIZE : uint = 1u << 32u;
const TCP_RECV_WND : uint = 1u << 10u;

pub struct TCB {

  // Buffers
  recv_buf : RingBuf, // TODO: abstraction over RingBuf needed?
  send_buf : RingBuf,

  // State Variables
  // TODO: sizes of all these
  send_UNA : uint,   // Oldest unacknowledged sequence number
  send_NXT : uint,   // Next Sequence Number to be Sent
  send_WND : uint,   // Send Window Size
  //send_UP  : uint, // send urgent pointer
  send_WL1 : uint,   // Seq number for last window update
  send_WL2 : uint,   // Ack number for last window update
  send_ISN : uint,   // Send Initial Sequence Number

  recv_NXT : uint,   // Expected next sequence number received
  recv_WND : uint,   // Recv Window Size
  //recv_UP  : uint, // Recv Urgent Pointer
  recv_ISN : uint,   // Recv Initial Sequence Number
}

impl TCB {
  pub fn new() -> TCB {
    TCB {
      recv_buf : RingBuf::new(TCP_BUF_SIZE),
      send_buf : RingBuf::new(TCP_BUF_SIZE),

      // TODO: How to initialize these?
      send_UNA : 0u,
      send_NXT : 0u,
      send_WND : 0u,
      send_WL1 : 0u,
      send_WL2 : 0u,
      send_ISN : 0u, //TODO: randomly generate

      recv_NXT : 0u,
      recv_WND : TCP_RECV_WND,
      recv_ISN : 0u, //TODO: randomly generate?
    }
  }
}
