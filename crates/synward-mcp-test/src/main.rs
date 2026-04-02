use std::process::Stdio;
use std::io::{BufRead, BufReader, Write};
use tokio::time::{sleep, Duration};
use std::thread;

#[tokio::main]
async fn main() {
    let server_path = r"C:\lex-exploratory\Synward\target\release\synward-mcp.exe";

    println!("Starting Synward MCP server test...");

    let mut child = std::process::Command::new(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    let stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    // Print stderr logs in background
    thread::spawn(move || {
        let mut stderr_reader = BufReader::new(stderr);
        let mut line = String::new();
        while let Ok(n) = stderr_reader.read_line(&mut line) {
            if n == 0 { break; }
            eprintln!("[LOG] {}", line.trim());
            line.clear();
        }
    });

    let mut stdin_writer = stdin;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Send initialize request
    let init_req = r#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}"#;
    println!("[IN] {}", init_req);
    writeln!(stdin_writer, "{}", init_req).expect("Failed to write init request");
    stdin_writer.flush().expect("Failed to flush");

    // Read response
    line.clear();
    reader.read_line(&mut line).expect("Failed to read");
    println!("[OUT] {}", line.trim());

    // Wait a bit
    sleep(Duration::from_millis(500)).await;

    // Send initialized notification
    let init_notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    println!("[IN] {}", init_notif);
    writeln!(stdin_writer, "{}", init_notif).expect("Failed to write init notification");
    stdin_writer.flush().expect("Failed to flush");

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    // Send tools/list request
    let tools_req = r#"{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}"#;
    println!("[IN] {}", tools_req);
    writeln!(stdin_writer, "{}", tools_req).expect("Failed to write tools request");
    stdin_writer.flush().expect("Failed to flush");

    // Read response
    sleep(Duration::from_millis(1000)).await;
    line.clear();
    match reader.read_line(&mut line) {
        Ok(0) => println!("[OUT] <EOF>"),
        Ok(_) => println!("[OUT] {}", line.trim()),
        Err(e) => println!("[OUT] Error: {}", e),
    }

    // Test tools/call - list_languages
    let call_req = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"list_languages","arguments":{}},"id":3}"#;
    println!("[IN] {}", call_req);
    writeln!(stdin_writer, "{}", call_req).expect("Failed to write call request");
    stdin_writer.flush().expect("Failed to flush");

    // Read response
    sleep(Duration::from_millis(500)).await;
    line.clear();
    match reader.read_line(&mut line) {
        Ok(0) => println!("[OUT] <EOF>"),
        Ok(_) => println!("[OUT] {}", line.trim()),
        Err(e) => println!("[OUT] Error: {}", e),
    }

    // Test tools/call - get_version
    let version_req = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_version","arguments":{}},"id":4}"#;
    println!("[IN] {}", version_req);
    writeln!(stdin_writer, "{}", version_req).expect("Failed to write version request");
    stdin_writer.flush().expect("Failed to flush");

    // Read response
    sleep(Duration::from_millis(500)).await;
    line.clear();
    match reader.read_line(&mut line) {
        Ok(0) => println!("[OUT] <EOF>"),
        Ok(_) => println!("[OUT] {}", line.trim()),
        Err(e) => println!("[OUT] Error: {}", e),
    }

    // Cleanup
    drop(stdin_writer); // Close stdin
    child.wait().expect("Failed to wait");
    println!("\n=== TEST COMPLETE ===");
}
