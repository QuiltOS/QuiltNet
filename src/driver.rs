use std::io;

struct IpCli;

/// Implementation of CLI that sits on IP module, querying its state
impl IpCli {

    /// Initiates REPL for manipulating and inspecting an IP module
    pub fn run(&self) -> () {
        
        // Read lines from stdin
        for line in io::stdin().lines() {
            match line {
                Ok(cmd) => {
                    
                    // Split line by whitespace, send args to dispatcher
                    let args : Vec<&str>= cmd.as_slice().trim().split(|c : char| c.is_whitespace()).collect();
                    self.handle_cmd(args.clone());
                },
                Err(e) => {

                    // is fail! the right move here?
                    fail!(e);
                }
            }
        }
    }

    /// Dispatch command to appropriate handler
    fn handle_cmd(&self, args : Vec<&str>) -> () {

        match args.as_slice() {

            /// Config inspectors
            ["interfaces"] => self.interfaces(),
            ["routes"] => self.routes(),
            
            /// Interface manipulators
            ["down", interface_s]   => {
                match from_str(interface_s){
                    Some(interface) => self.down(interface),
                    None    => self.invalid(args)
                }
            },
            ["up", interface_s]   => {
                match from_str(interface_s){
                    Some(interface) => self.up(interface),
                    None    => self.invalid(args)
                }
            },

            /// Sends data to given Virtual IP address using designated protocol
            ["send", vip, proto_s, data] => {
                match from_str(proto_s) {
                    Some(proto) => self.send(vip, proto, data),
                    None    => self.invalid(args)
                }
            }

            /// Catch-all for invalid commands
            _ => self.invalid(args)
        }
    }

    /// TODO: Send data to given Virtual IP address using designated protocol
    fn send(&self, vip : &str, proto : int, data : &str){
        println!("Sending {}, {}, {}", vip, proto, data);
    }

    /// TODO: Bring interface back up if it is down
    fn up(&self, interface : int){
        println!("Up {}", interface);
    }

    /// TODO: Bring interface down if it is up
    fn down(&self, interface : int){
        println!("Down {}", interface);
    }

    /// TODO: List all interfaces the IP module exposes
    fn interfaces(&self){
        println!("Interfaces!");
    }

    /// TODO: List all routes in the IP module's routing table
    fn routes(&self){
        println!("Routes!");
    }

    /// Indicates invalid REPL command 
    /// TODO: add string msg argument
    fn invalid(&self, args: Vec<&str>){
        println!("Invalid Command! {}", args);
    }
}

/// Test out REPL without any IP module underneath
fn main(){
    let node = IpCli;
    node.run();
}
