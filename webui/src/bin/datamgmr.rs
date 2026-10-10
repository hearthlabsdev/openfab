use ormlite::{model::Model, postgres::PgPool};
use prusa_rs::settings::{
    PrusaIni,
    filaments::Filament,
    filaments::{FilamentIdentity, FilamentType},
    printers::PrinterModel,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use structopt::StructOpt;
use uuid::Uuid;
/*
structopt::clap::arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum SettingsProfile {
        DeviceModels,
        Filament
    }
}*/

// Adjust these imports to wherever these live in your project.
use webui::{
    devices::DeviceModelORM,
    library::{AssetORM, LibraryORM, ObjectStore},
    settings::materials::FilamentORM,
};

const PRUSA_SETTINGS_LICENSE: &str = "AGPL-3.0";

#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    /// database str e.g. postgres://openfab:password@localhost:5432/openfab
    #[structopt(short, long)]
    pub database: String,

    /// load printer models
    #[structopt(short, long)]
    pub prusa_settings: Option<PathBuf>,
    #[structopt(long)]
    pub profile: Option<String>,
    /// MinIO/S3 endpoint
    #[structopt(long, default_value = "localhost:3900")]
    pub store_endpoint: String,

    /// MinIO/S3 access key
    #[structopt(long)]
    pub store_key: String,

    /// MinIO/S3 secret key
    #[structopt(long)]
    pub store_secret: String,

    /// MinIO/S3 bucket
    #[structopt(long, default_value = "openfab")]
    pub store_bucket: String,

    /// Use HTTPS when connecting to object storage
    #[structopt(long)]
    pub store_secure: bool,

    /// library to upload assets to, defaults to creating a new library.
    #[structopt(short, long)]
    library: Option<Uuid>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let args = Args::from_args();

    let pool = PgPool::connect(&args.database)
        .await
        .expect("failed to create database pool");

    let object_store = ObjectStore::new(
        &args.store_endpoint,
        &args.store_key,
        &args.store_secret,
        &args.store_bucket,
        args.store_secure,
    )?;

    let Some(dir) = args.prusa_settings else {
        return Ok(());
    };

    let mut conn = pool.acquire().await.unwrap();
    let library = match args.library {
        Some(library) => LibraryORM::select()
            .where_("uid = ?")
            .bind(library)
            .fetch_one(&mut *conn)
            .await
            .expect("failed to fetch existing library"),
        None => LibraryORM::new("prusa-settings-library", Uuid::nil(), true)
            .insert(&mut *conn)
            .await
            .expect("failed to create new library"),
    };

    let ini = PrusaIni::import_live(&dir).await.unwrap();

    /*
     * First collect all printer models.
     *
     * We use the model name as the canonical identity rather than the
     * section ID because the same model can occur in multiple vendor
     * bundles/versions.
     */
    let mut models: HashMap<String, PrinterModel> = HashMap::new();
    let mut filaments = Vec::new();
    for (_vendor, data) in ini.into_iter() {
        for (_version, sections) in data.into_iter() {
            for section in sections.sections.into_iter() {
                if section.kind != "printer_model" && section.kind != "filament" {
                    continue;
                }
                if args.profile == Some("filament".to_string()) {
                    let model = Filament::from_ini_section(section)?;
                    let identity = model.identity;
                    println!("identity: {:?}", identity);
                    filaments.push(identity);
                } else if args.profile == Some("printer_models".to_string()) {
                    let model = PrinterModel::from_ini_section(section)?;

                    models
                        .entry(model.name.clone())
                        .and_modify(|existing| merge_printer_models(existing, &model))
                        .or_insert(model);
                }
            }
        }
    }

    println!("found {} unique printer models", models.len(),);
    let filaments = deduplicate_filament_profiles(filaments);
    for filament in filaments.into_iter() {
        import_filament_profile(&pool, library.uid, filament)
            .await
            .unwrap();
    }
    /*
     * Import the canonical models.
     */
    for (_, model) in models {
        import_printer_model(&pool, &object_store, &dir, library.uid, model).await?;
    }

    Ok(())
}

pub fn filament_type_to_string(filament_type: FilamentType) -> String {
    match filament_type {
        FilamentType::ABS => "ABS".into(),
        FilamentType::ASA => "ASA".into(),
        FilamentType::CPE => "CPE".into(),
        FilamentType::EDGE => "EDGE".into(),
        FilamentType::FLEX => "FLEX".into(),
        FilamentType::HIPS => "HIPS".into(),
        FilamentType::NGEN => "nGen".into(),
        FilamentType::Nylon => "Nylon".into(),
        FilamentType::PA => "PA".into(),
        FilamentType::PEBA => "PEBA".into(),
        FilamentType::PC => "PC".into(),
        FilamentType::PET => "PET".into(),
        FilamentType::PETG => "PETG".into(),
        FilamentType::PLATough => "PLA Tough".into(),
        FilamentType::PCTG => "PCTG".into(),
        FilamentType::PP => "PP".into(),
        FilamentType::PVA => "PVA".into(),
        FilamentType::PVB => "PVB".into(),
        FilamentType::PLA => "PLA".into(),
        FilamentType::TPU => "TPU".into(),
        FilamentType::PEI => "PEI".into(),
        FilamentType::Metal => "Metal".into(),
        FilamentType::Glaze => "Glaze".into(),
        FilamentType::Other(value) => value.into(),
    }
}

fn deduplicate_filament_profiles(profiles: Vec<FilamentIdentity>) -> Vec<FilamentIdentity> {
    let mut deduplicated = Vec::with_capacity(profiles.len());

    for profile in profiles {
        if !deduplicated.iter().any(|existing: &FilamentIdentity| {
            existing.vendor == profile.vendor && existing.name == profile.name
        }) {
            deduplicated.push(profile);
        }
    }

    deduplicated
}

async fn import_filament_profile(
    pool: &PgPool,
    library: Uuid,
    profile: FilamentIdentity,
) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;
    let filament = FilamentORM {
        uid: Uuid::new_v4(),
        name: profile.name,
        vendor: profile.vendor,
        cost: profile.cost.map(|v| v as f64),
        colour: profile.colour,
        density: profile
            .density
            .map(|v| format!("{}, {}", v.left, v.right.unwrap_or(0.0))),
        spool_weight: profile.spool_weight.map(|v| v.to_string()),
        filament_type: profile.filament_type.map(|v| filament_type_to_string(v)),
        notes: profile.notes,
    };
    filament.insert(&mut *conn).await?;
    Ok(())
}

async fn import_printer_model(
    pool: &PgPool,
    object_store: &ObjectStore,
    settings_dir: &Path,
    library: Uuid,
    model: PrinterModel,
) -> anyhow::Result<()> {
    let uid = Uuid::new_v4();

    let bed_model = if let Some(path) = model.bed_model.as_deref() {
        import_asset(pool, object_store, settings_dir, path, library, "model").await?
    } else {
        None
    };

    let bed_texture = if let Some(path) = model.bed_texture.as_deref() {
        import_asset(pool, object_store, settings_dir, path, library, "texture").await?
    } else {
        None
    };

    let thumbnail = if let Some(path) = model.thumbnail.as_deref() {
        import_asset(pool, object_store, settings_dir, path, library, "thumbnail").await?
    } else {
        None
    };

    let device_model = DeviceModelORM {
        uid,
        name: model.name,
        variants: model.variants,
        technology: model.technology,
        family: model.family,
        bed_model,
        bed_texture,
        thumbnail: thumbnail,
        default_materials: model.default_materials,
    };

    device_model.insert(pool).await?;

    Ok(())
}

async fn import_asset(
    pool: &PgPool,
    object_store: &ObjectStore,
    settings_dir: &Path,
    filename: &str,
    library: Uuid,
    asset_kind: &str,
) -> anyhow::Result<Option<Uuid>> {
    let path = match find_file(settings_dir, filename).await? {
        Some(path) => path,
        None => return Ok(None),
    };

    let data = tokio::fs::read(&path).await?;

    let uid = Uuid::new_v4();

    let key = format!(
        "prusa-settings/device-models/{}/{}/{}",
        asset_kind, uid, filename
    );

    object_store.upload_bytes(&key, data.clone()).await?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    AssetORM {
        uid,
        name: filename.to_owned(),
        key,
        library,
        mime: mime_for_filename(filename),
        owner: Uuid::nil(),
        size: data.len() as i64,
        created_at: now,
        updated_at: now,
        last_used_at: None,
        public: true,
        license: Some(PRUSA_SETTINGS_LICENSE.to_owned()),
    }
    .insert(pool)
    .await?;

    Ok(Some(uid))
}

async fn find_file(root: &Path, filename: &str) -> anyhow::Result<Option<PathBuf>> {
    let mut dir = tokio::fs::read_dir(root).await?;

    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();

        if path.is_file() {
            if path.file_name().and_then(|x| x.to_str()) == Some(filename) {
                return Ok(Some(path));
            }

            continue;
        }

        if path.is_dir() {
            if let Some(found) = Box::pin(find_file(&path, filename)).await? {
                return Ok(Some(found));
            }
        }
    }

    Ok(None)
}

fn mime_for_filename(filename: &str) -> String {
    match Path::new(filename)
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_ascii_lowercase())
        .as_deref()
    {
        Some("stl") => "model/stl",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
    .to_owned()
}

fn merge_printer_models(existing: &mut PrinterModel, incoming: &PrinterModel) {
    for variant in &incoming.variants {
        if !existing.variants.contains(variant) {
            existing.variants.push(variant.clone());
        }
    }

    for material in &incoming.default_materials {
        if !existing.default_materials.contains(material) {
            existing.default_materials.push(material.clone());
        }
    }

    if existing.technology.is_none() {
        existing.technology = incoming.technology.clone();
    }

    if existing.family.is_none() {
        existing.family = incoming.family.clone();
    }

    if existing.bed_model.is_none() {
        existing.bed_model = incoming.bed_model.clone();
    }

    if existing.bed_texture.is_none() {
        existing.bed_texture = incoming.bed_texture.clone();
    }
}
