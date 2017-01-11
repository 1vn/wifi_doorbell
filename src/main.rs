use std::process::Command;
use std::string::String;
use std::thread;
use std::time::Duration;

fn exec_ping_ips_by_range(start: u32, end: u32){
	for i in start..end {
		if i < 1 || i == 255 {
			return;
		}

		Command::new("ping")
			.arg("-c 1")
			.arg("-W 0.1")
			.arg(format!("192.168.0.{}", i))
			.output()
			.expect("failed to execute ping");
		println!("pinging: 192.168.0.{}", i);
	}

}

fn refresh_arp_cache() {
	Command::new("arp")
			.arg("-a")
			.arg("-d")
			.output()
			.expect("failed to execute refresh");
}

struct Device {
	ip: String,
	mac: String
}

fn get_current_devices() -> Vec<Device>{
	let mut current_devices :Vec<Device> = Vec::new();
	
	let output = Command::new("arp")
						 .arg("-a")
						 .output()
						 .expect("failed to execute fetch all");
	let s = String::from_utf8(output.stdout).unwrap();
	let lines: Vec<&str> = s.split("\n").collect();
	 	
	for line in lines {
		if line.contains("(incomplete)") || line.chars().count() == 0{
			continue;
		}

		let cols: Vec<&str>  = line.split_whitespace().collect();
		let ip = cols[1];
		let mac = cols[3];
		current_devices.push(Device{ip: ip.to_string(), mac: mac.to_string()});
		println!("{} {}", ip, mac);	
	}

	return current_devices;
}

fn main() {
	// 1. arp -a -d to refresh
	// 2. ping all possible addr
    let mut children = vec![];
	for i in 1..52 {
		children.push(thread::spawn(move ||{
				exec_ping_ips_by_range((i-1) * 5, i * 5);
		}));
	}


	for child in children {
    	// Wait for the thread to finish. Returns a result.
    	let _ = child.join();
	}

	// 3. arp -a get list of currently connected devices
	let mut current_devices :Vec<Device> = get_current_devices();
	
	loop {
		// 4. wait 30s
		thread::sleep(Duration::from_millis(10000));
		
		// 5. arp -a -d to refresh
		let mut children = vec![];
		for i in 1..51 {
			children.push(thread::spawn(move ||{
				exec_ping_ips_by_range((i-1) * 5, i * 5);
			}));
		}

		for child in children {
    		// Wait for the thread to finish. Returns a result.
    		let _ = child.join();
		}

		// 6. compare against list of currently connected devices, notify additions and update list of currently connected devices
		let check_devices = get_current_devices();
		if check_devices.len() != current_devices.len() {
			println!("!-- change alert --!");
			if check_devices.len() > current_devices.len() {
				println!("!-- {} new devices --!", check_devices.len() - current_devices.len())
			} else {
				println!("!-- {} devices disconnected--!", current_devices.len() - check_devices.len())
			}
			current_devices = check_devices;
		}

	}
}
