use anyhow::Result;
use certify::{generate_ca, generate_cert, CertType, CA, load_ca};
use tokio::fs;

struct CertPem {
    cert_type: CertType,
    cert: String,
    key: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pem = create_ca()?;
    gen_files(&pem).await?;
    let ca = load_ca(&pem.cert, &pem.key)?;
    let pem = create_cert(&ca, &["avrilko.com"], "avrilko", true)?;
    gen_files(&pem).await?;

    let pem = create_cert(&ca, &["avrilko.com"], "avrilko", false)?;
    gen_files(&pem).await?;
    Ok(())
}

fn create_ca() -> Result<CertPem> {
    let (cert, key) = generate_ca(
        &["avrilko.com"],
        "CN",
        "avrilko",
        "avrilko",
        None,
        Some(10 * 365),
    )?;

    Ok(CertPem {
        cert_type: CertType::CA,
        cert,
        key,
    })
}

fn create_cert(ca: &CA, domain: &[&str], cn: &str, is_client: bool) -> Result<CertPem> {
    let (day, cert_type) = match is_client {
        true => (Some(365), CertType::Client),
        false => (Some(365 * 10), CertType::Server),
    };

    let (cert, key) = generate_cert(ca, domain, "CN", "avrilko", cn, None, is_client, day)?;

    Ok(CertPem {
        cert_type,
        cert,
        key,
    })
}

async fn gen_files(pem: &CertPem) -> Result<()> {
    let name = match &pem.cert_type {
        CertType::Client => "client",
        CertType::Server => "server",
        CertType::CA => "ca",
    };
    fs::write(format!("fixtures/{}.cert", name), &pem.cert).await?;
    fs::write(format!("fixtures/{}.key", name), &pem.key).await?;
    Ok(())
}
