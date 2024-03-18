use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = gc_rs::get_matches();
    gc_rs::handle(matches).await
}
