package tcp.packet;

import ip.packet.IpPacket;

import java.util.Arrays;

import tcp.connect.ConnectionId;
import tcp.connect.SockAddr;
import util.Utils;

import common.Ipv4Addr;

public class TcpPacket {

	// TCP header w/o options is 5 32-bit words
	private static int TCP_HDR_LEN = 5 * 4;

	private Ipv4Addr srcAddr;
	private Ipv4Addr dstAddr;
	private int protocol;
	private int tcpLen;
	private final byte[] header;
	private byte[] payload;

	// Constructor for receiving TCP packets
	//FIXME: for speed, just wrap same data buffer instead of allocate-copy
	public TcpPacket(IpPacket ip){

		// Save IP parameters for checksumming
		this.srcAddr  = ip.getSource();
		this.dstAddr  = ip.getDestination();
		this.protocol = ip.getProtocol();
		this.tcpLen   = ip.getDataLength();

		// Allocate and copy TCP header from IP payload
		this.header = new byte[TCP_HDR_LEN];
		byte[] ipdata = ip.getPayload();
		System.arraycopy(ipdata, 0, this.header, 0, TCP_HDR_LEN);

		// Allocate and copy TCP payload from IP payload
		this.payload = new byte[ipdata.length - TCP_HDR_LEN];
		System.arraycopy(ipdata, TCP_HDR_LEN, this.payload, 0, ipdata.length - TCP_HDR_LEN);

	}

	// Constructor for sending TCP packets
	public TcpPacket(int srcPort, int dstPort, int seqNum, int ackNum, int window, byte[] data){

		// Allocate header and set with data
		this.header = new byte[TCP_HDR_LEN];
		setSrcPort(srcPort);
		setDstPort(dstPort);
		setSequenceNum(seqNum);
		setAckNum(ackNum);
		setWindow(window);

		// Should we copy?
		this.payload = data;
	}

	// Get Connection this packet's addresses represent
	public ConnectionId getConnectionId(){
		return new ConnectionId(new SockAddr(srcAddr, getSrcPort()), new SockAddr(dstAddr, getDstPort()));
	}

	// Manipulate Source Port
	public void setSrcPort(int port){
		Utils.setMultiByte(port, header, 0, 1);
	}
	public int getSrcPort(){
		return Utils.getMultiByte(header, 0, 1);
	}

	// Manipulate Destination Port
	public void setDstPort(int port){
		Utils.setMultiByte(port, header, 2, 3);
	}
	public int getDstPort(){
		return Utils.getMultiByte(header, 2, 3);
	}

	// Manipulate Sequence Number
	public void setSequenceNum(int seqNum){
		Utils.setMultiByte(seqNum, header, 4, 5, 6, 7);
	}
	public int getSequenceNum(){
		return Utils.getMultiByte(header, 4, 5, 6, 7);
	}

	// Manipulate Acknowledgement Number
	public void setAckNum(int ackNum){
		Utils.setMultiByte(ackNum, header, 8, 9, 10, 11);
	}
	public int getAckNum(){
		return Utils.getMultiByte(header, 8, 9, 10, 11);
	}

	/************************************************/
	// Idgaf: weird alignment shit
	public int getDataOffset(){
		return (header[12] & 0xFF) >> 4;
	}

	// Is URG control flag set
	public boolean isURG(){
		return (header[13] & 32) != 0;
	}

	// Is ACK control flag set
	public boolean isACK(){
		return (header[13] & 16) != 0;
	}

	// Is PSH control flag set
	public boolean isPSH(){
		return (header[13] & 8) != 0;
	}

	// Is RST control flag set
	public boolean isRST(){
		return (header[13] & 4) != 0;
	}

	// Is SYN control flag set
	public boolean isSYN(){
		return (header[13] & 2) != 0;
	}

	// Is FIN control flag set
	public boolean isFIN(){
		return (header[13] & 1) != 0;
	}
	/************************************************/

	// Manipulate Current Window
	public void setWindow(int window){
		Utils.setMultiByte(window, header, 14, 15);
	}
	public int getWindow(){
		return Utils.getMultiByte(header,  14, 15);
	}

	// Manipulate Checksum
	public void setChecksum(int checksum){
		Utils.setMultiByte(checksum, header, 16, 17);
	}
	public int getChecksum(){
		return Utils.getMultiByte(header,  16, 17);
	}
	// Verify that computed checksum is checksum on record
	public boolean validate() {
		return computeChecksum() == getChecksum();
	}
	// Compute TCP checksum using TCP header and stored IP data
	public int computeChecksum(){
		//TODO
		System.err.println("TODO: compute checksum");
		return 0;
	}
	
	// Should we copy?
	public void setData(byte[] _data){
		this.payload = _data;
	}
	public byte[] getData(){
		return this.payload;
	}

	@Override
	public String toString(){
		return String.format("|SrcPort: %s|DstPort: %s|\n"
				+ "|Seq#: %s|\n"
				+ "|Ack#: %s|\n"
				+ "|Offset: %s| Reserved |URG: %s|ACK: %s|PSH: %s|RST: %s|SYN: %s|FIN: %s|Window: %s|\n"
				+ "|Checksum: %s|UrgPtr: %s|\n"
				+ "|Data: %s|", getSrcPort(), getDstPort(),
				getSequenceNum(),
				getAckNum(),
				getDataOffset(), isURG(), isACK(), isPSH(), isRST(), isSYN(), isFIN(), getWindow(),
				getChecksum(), -2,
				getData());
	}




}
