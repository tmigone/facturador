//! `arquita` — issue Argentine electronic invoices (Factura C) against
//! ARCA/AFIP from the command line.

use std::path::{Path, PathBuf};

use afip::{
    Client, CondicionIva, Concepto, DocTipo, EmisorConfig, Environment, FacturaC, VoucherType,
};
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "arquita", version, about = "Facturación electrónica ARCA/AFIP (Factura C)")]
struct Cli {
    /// Working directory holding config, certs and the credential cache.
    /// Defaults to $AFIP_HOME, then ~/arquita.
    #[arg(long, global = true)]
    home: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the issuer config (emisor_config.json).
    Init {
        #[arg(long)]
        cuit: u64,
        #[arg(long)]
        punto_venta: u32,
        #[arg(long)]
        razon_social: String,
        #[arg(long, value_enum, default_value_t = CondicionIvaArg::Monotributo)]
        condicion_iva: CondicionIvaArg,
        /// Target the real environment (default: homologación/testing).
        #[arg(long)]
        production: bool,
    },
    /// Generate the private key + CSR to upload to the ARCA portal.
    GenerateCertificate {
        /// Certificate alias / common name.
        #[arg(long, default_value = "arquita")]
        alias: String,
        /// Overwrite an existing key/CSR.
        #[arg(long)]
        force: bool,
    },
    /// Check WSFE service health (FEDummy).
    Status,
    /// Print the last authorized Factura C number.
    LastVoucher,
    /// Issue a Factura C and print the CAE.
    CreateVoucher {
        /// Total amount in pesos.
        #[arg(long)]
        importe: f64,
        #[arg(long, value_enum, default_value_t = ConceptoArg::Productos)]
        concepto: ConceptoArg,
        #[arg(long, value_enum, default_value_t = DocTipoArg::ConsumidorFinal)]
        doc_tipo: DocTipoArg,
        /// Receptor document number (0 for consumidor final).
        #[arg(long, default_value_t = 0)]
        doc_nro: u64,
        /// Receptor condición IVA id (5 = consumidor final).
        #[arg(long, default_value_t = 5)]
        cond_iva_receptor: u8,
        /// Voucher date, YYYYMMDD (default: today, AR timezone).
        #[arg(long)]
        fecha: Option<u32>,
        /// Service period start, YYYYMMDD (services only; default: fecha).
        #[arg(long)]
        fecha_servicio_desde: Option<u32>,
        /// Service period end, YYYYMMDD (services only; default: fecha).
        #[arg(long)]
        fecha_servicio_hasta: Option<u32>,
        /// Payment due date, YYYYMMDD (services only; default: fecha).
        #[arg(long)]
        fecha_vto_pago: Option<u32>,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum CondicionIvaArg {
    Monotributo,
    ResponsableInscripto,
    Exento,
}

impl From<CondicionIvaArg> for CondicionIva {
    fn from(v: CondicionIvaArg) -> Self {
        match v {
            CondicionIvaArg::Monotributo => CondicionIva::Monotributo,
            CondicionIvaArg::ResponsableInscripto => CondicionIva::ResponsableInscripto,
            CondicionIvaArg::Exento => CondicionIva::Exento,
        }
    }
}

#[derive(Copy, Clone, ValueEnum)]
enum ConceptoArg {
    Productos,
    Servicios,
    ProductosYServicios,
}

impl From<ConceptoArg> for Concepto {
    fn from(v: ConceptoArg) -> Self {
        match v {
            ConceptoArg::Productos => Concepto::Productos,
            ConceptoArg::Servicios => Concepto::Servicios,
            ConceptoArg::ProductosYServicios => Concepto::ProductosYServicios,
        }
    }
}

#[derive(Copy, Clone, ValueEnum)]
enum DocTipoArg {
    Cuit,
    Cuil,
    Dni,
    ConsumidorFinal,
}

impl From<DocTipoArg> for DocTipo {
    fn from(v: DocTipoArg) -> Self {
        match v {
            DocTipoArg::Cuit => DocTipo::Cuit,
            DocTipoArg::Cuil => DocTipo::Cuil,
            DocTipoArg::Dni => DocTipo::Dni,
            DocTipoArg::ConsumidorFinal => DocTipo::ConsumidorFinal,
        }
    }
}

fn home_dir(cli: &Cli) -> PathBuf {
    if let Some(h) = &cli.home {
        return h.clone();
    }
    if let Ok(h) = std::env::var("AFIP_HOME") {
        return PathBuf::from(h);
    }
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("arquita")
}

fn config_path(home: &Path) -> PathBuf {
    home.join("emisor_config.json")
}

fn load_client(home: &Path) -> Result<Client> {
    let cfg = EmisorConfig::load(config_path(home))
        .context("could not load emisor_config.json — run `arquita init` first")?;
    let client = Client::new(cfg, home.join("cache"))?;
    Ok(client)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let home = home_dir(&cli);

    match &cli.command {
        Command::Init {
            cuit,
            punto_venta,
            razon_social,
            condicion_iva,
            production,
        } => cmd_init(
            &home,
            *cuit,
            *punto_venta,
            razon_social,
            (*condicion_iva).into(),
            *production,
        ),
        Command::GenerateCertificate { alias, force } => cmd_generate_cert(&home, alias, *force),
        Command::Status => cmd_status(&home),
        Command::LastVoucher => cmd_last_voucher(&home),
        Command::CreateVoucher {
            importe,
            concepto,
            doc_tipo,
            doc_nro,
            cond_iva_receptor,
            fecha,
            fecha_servicio_desde,
            fecha_servicio_hasta,
            fecha_vto_pago,
        } => {
            let factura = FacturaC {
                concepto: (*concepto).into(),
                doc_tipo: (*doc_tipo).into(),
                doc_nro: *doc_nro,
                importe_total: *importe,
                fecha: *fecha,
                fecha_servicio_desde: *fecha_servicio_desde,
                fecha_servicio_hasta: *fecha_servicio_hasta,
                fecha_vto_pago: *fecha_vto_pago,
                condicion_iva_receptor: *cond_iva_receptor,
            };
            cmd_create_voucher(&home, factura)
        }
    }
}

fn cmd_init(
    home: &Path,
    cuit: u64,
    punto_venta: u32,
    razon_social: &str,
    condicion_iva: CondicionIva,
    production: bool,
) -> Result<()> {
    std::fs::create_dir_all(home.join("certs"))?;
    let cfg = EmisorConfig {
        cuit,
        punto_venta,
        razon_social: razon_social.to_string(),
        condicion_iva,
        environment: if production {
            Environment::Produccion
        } else {
            Environment::Homologacion
        },
        cert_path: home.join("certs/arquita.crt"),
        key_path: home.join("certs/arquita.key"),
    };
    let path = config_path(home);
    cfg.save(&path)?;
    println!("✔ Wrote {}", path.display());
    println!(
        "  environment: {}",
        if production { "producción" } else { "homologación" }
    );
    println!("  Next: `arquita generate-certificate`");
    Ok(())
}

fn cmd_generate_cert(home: &Path, alias: &str, force: bool) -> Result<()> {
    let cfg = EmisorConfig::load(config_path(home)).context("run `arquita init` first")?;

    let certs_dir = home.join("certs");
    std::fs::create_dir_all(&certs_dir)?;
    let key_path = certs_dir.join(format!("{alias}.key"));
    let csr_path = certs_dir.join(format!("{alias}.csr"));

    if key_path.exists() && !force {
        bail!(
            "{} already exists — pass --force to overwrite (this invalidates the ARCA-issued cert)",
            key_path.display()
        );
    }

    let out = afip::cert::generate_key_and_csr(cfg.cuit, &cfg.razon_social, alias)?;
    std::fs::write(&key_path, out.private_key_pem)?;
    std::fs::write(&csr_path, out.csr_pem)?;

    println!("✔ Private key: {}", key_path.display());
    println!("✔ CSR:         {}", csr_path.display());
    println!();
    println!("Next steps (manual, one time):");
    println!("  1. Log into the ARCA portal → «Administración de Certificados Digitales».");
    println!("  2. Upload {}.", csr_path.display());
    println!("  3. Download the issued certificate to {}.", cfg.cert_path.display());
    println!("  4. Associate the cert with «Facturación Electrónica» (WSFE) in «Administrador de Relaciones».");
    Ok(())
}

fn cmd_status(home: &Path) -> Result<()> {
    let client = load_client(home)?;
    let (app, db, auth) = client.status()?;
    println!("WSFE status → AppServer: {app} | DbServer: {db} | AuthServer: {auth}");
    Ok(())
}

fn cmd_last_voucher(home: &Path) -> Result<()> {
    let client = load_client(home)?;
    let n = client.last_voucher(VoucherType::FacturaC)?;
    println!("Last authorized Factura C: {n} (next: {})", n + 1);
    Ok(())
}

fn cmd_create_voucher(home: &Path, factura: FacturaC) -> Result<()> {
    if factura.importe_total <= 0.0 {
        bail!("importe must be positive");
    }
    let client = load_client(home)?;
    let res = client.create_factura_c(&factura)?;
    println!("✔ Factura C autorizada");
    println!("  Punto de venta: {:04}", res.punto_venta);
    println!("  Número:         {:08}", res.numero);
    println!("  Fecha:          {}", res.fecha);
    println!("  Importe total:  ${:.2}", res.importe_total);
    println!("  CAE:            {}", res.cae);
    println!("  CAE vto:        {}", res.cae_vencimiento);
    if !res.observaciones.is_empty() {
        println!("  Observaciones:");
        for o in &res.observaciones {
            println!("    - {o}");
        }
    }
    Ok(())
}
