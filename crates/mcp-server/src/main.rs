//! Transport stdio uniquement : stdout appartient exclusivement au protocole MCP.
use rmcp::{ServiceExt, transport::async_rw::AsyncRwTransport};
use systempulse_mcp_server::MonitorServer;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("SystemPulse MCP : {error}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), String> {
    let mut share_processes = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--share-processes" => share_processes = true,
            "--help" => {
                eprintln!("systempulse-mcp [--share-processes] — MCP local stdio, lecture seule");
                return Ok(());
            }
            _ => return Err("Option inconnue ; consulter --help".into()),
        }
    }
    let (server, _guard) = MonitorServer::start(share_processes)?;
    let transport = AsyncRwTransport::new(tokio::io::stdin(), tokio::io::stdout())
        .with_max_line_length(64 * 1024);
    let running = tokio::time::timeout(std::time::Duration::from_secs(15), server.serve(transport))
        .await
        .map_err(|_| "Délai de connexion dépassé")?
        .map_err(|_| "Connexion MCP impossible")?;
    running
        .waiting()
        .await
        .map_err(|_| "Transport MCP interrompu")?;
    Ok(())
}
