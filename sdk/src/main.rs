use openfab_sdk::{errors::SDKError, files::FileTypeChecker, Configuration};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "openfab-cli",
    about = "OpenFAB command-line client",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    /// Operation to perform on a resource.
    #[structopt(subcommand)]
    pub command: Command,

    /// Optional request arguments in KEY=VALUE form.
    ///
    /// Example:
    ///   --arg=id=123
    ///   --arg=name=demo
    ///   --arg=limit=50
    #[structopt(long = "arg", parse(try_from_str = parse_key_value))]
    pub args: Vec<KeyValueArg>,

    /// Optional custom User-Agent string to send with HTTP requests.
    #[structopt(long, env = "OPENFAB_USER_AGENT", default_value = "OpenFAB Cli")]
    pub user_agent: String,

    #[structopt(
        short,
        long,
        env = "OPENFAB_HOST",
        default_value = "http://localhost:7777"
    )]
    pub host: String,

    /// Optional API key used for authentication.
    #[structopt(long, env = "OPENFAB_API_KEY")]
    pub api_key: String,

    /// by default the API implements pagination, use this to select a page.
    #[structopt(short, long)]
    pub page: Option<i64>,

    /// a filter used to narrow down models, for example when listing assets this would be the Unique-ID of the Library they are in.
    #[structopt(long)]
    pub id: Option<String>,

    #[structopt(short, long)]
    pub file: Option<PathBuf>,

    #[structopt(short, long)]
    pub body: Option<String>,
}

impl Cli {
    pub async fn process(&self, config: Configuration) -> Result<(), SDKError> {
        match &self.command {
            Command::List { model } => {
                Command::list(&model, config, self.id.clone(), self.page).await?;
            }
            Command::Get { model } => {
                Command::get(
                    &model,
                    config,
                    self.id
                        .clone()
                        .ok_or(SDKError::MissingArgument("uid".into()))?,
                )
                .await?
            }
            Command::Create { model } => {
                Command::create(
                    &model,
                    config,
                    self.body
                        .clone()
                        .ok_or(SDKError::MissingArgument("body".into()))?,
                    self.file.clone(),
                )
                .await?;
            },
            Command::Delete { model } => {
                Command::delete(&model, config, self.id.clone().ok_or(SDKError::MissingArgument("id".into()))?).await?;
            },
            _ => unimplemented!(),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, StructOpt)]
pub enum Command {
    /// List resources
    #[structopt(name = "list")]
    List {
        /// Resource type to list
        #[structopt(subcommand)]
        model: ResourceModel,
    },

    /// Get a specific resource
    #[structopt(name = "get")]
    Get {
        /// Resource type to get
        #[structopt(subcommand)]
        model: ResourceModel,
    },

    /// Create a new resource
    #[structopt(name = "create")]
    Create {
        /// Resource type to create
        #[structopt(subcommand)]
        model: ResourceModel,
    },

    /// Update an existing resource
    #[structopt(name = "update")]
    Update {
        /// Resource type to update
        #[structopt(subcommand)]
        model: ResourceModel,
    },

    /// Delete a resource
    #[structopt(name = "delete")]
    Delete {
        /// Resource type to delete
        #[structopt(subcommand)]
        model: ResourceModel,
    },
}

impl Command {
    pub async fn list(
        model: &ResourceModel,
        config: Configuration,
        id: Option<String>,
        page: Option<i64>,
    ) -> Result<(), SDKError> {
        match model {
            ResourceModel::Devices => {
                let resp = config
                    .list_devices(page)
                    .await
                    .expect("failed to list devices");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).expect("failed to serialize response")
                );
            }
            ResourceModel::Libraries => {
                let resp = config
                    .list_libraries()
                    .await
                    .expect("failed to list libraries");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).expect("failed to serialize response")
                );
            }
            ResourceModel::Materials => {
                //let resp = config.list_materials(page).await.expect("failed to list devices");
                //println!("{}", serde_json::to_string_pretty(&resp).expect("failed to serialize response"));
            }
            ResourceModel::Assets => {
                let resp = config
                    .list_assets(
                        &id.ok_or(SDKError::MissingArgument("library".into()))?,
                        page,
                    )
                    .await
                    .expect("failed to list devices");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).expect("failed to serialize response")
                );
            }
            _ => unimplemented!(),
        };
        Ok(())
    }

    pub async fn get(
        model: &ResourceModel,
        config: Configuration,
        id: String,
    ) -> Result<(), SDKError> {
        match model {
            ResourceModel::Devices => {
                let resp = config.get_device(&id).await.expect("failed to get device");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).expect("failed to serialize response")
                );
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
    pub async fn create(
        model: &ResourceModel,
        config: Configuration,
        body: String,
        file: Option<PathBuf>,
    ) -> Result<(), SDKError> {
        let file_checker = FileTypeChecker::default_checker();
        match model {
            ResourceModel::Assets => {
                let path = file.ok_or(SDKError::MissingArgument("file".into()))?;
                let mime_type = file_checker.detect_for_file(&path).await.expect("failed to get file info").mime;
                let file = tokio::fs::File::open(path)
                        .await?;
                let asset = serde_json::from_str(&body).expect("failed to deserialize asset");
                let resp = config.create_asset(&asset, file, mime_type.to_string()).await?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).expect("failed to deserialize response")
                );
            }
            _ => {}
        }
        Ok(())
    }
    pub async fn delete(model: &ResourceModel, config: Configuration, id: String) -> Result<(), SDKError> {
        match model {
            ResourceModel::Assets => {
                let resp = config.delete_asset(&id).await?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).expect("failed to deserialize response")
                );
            },
            _ => unimplemented!(),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, StructOpt)]
pub enum ResourceModel {
    #[structopt(name = "queues")]
    Queues,
    #[structopt(name = "jobs")]
    Jobs,
    #[structopt(name = "devices")]
    Devices,
    #[structopt(name = "device-models")]
    DeviceModels,
    #[structopt(name = "materials")]
    Materials,
    #[structopt(name = "libraries")]
    Libraries,
    #[structopt(name = "assets")]
    Assets,
}

#[derive(Debug, Clone)]
pub struct KeyValueArg {
    pub key: String,
    pub value: String,
}

impl FromStr for KeyValueArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '=');

        let key = parts
            .next()
            .ok_or_else(|| "argument must be in KEY=VALUE form".to_string())?
            .trim();

        let value = parts.next().unwrap_or("").trim().to_string();

        if key.is_empty() {
            return Err("key cannot be empty".to_string());
        }

        Ok(Self {
            key: key.to_string(),
            value,
        })
    }
}

impl Cli {
    pub fn args_as_map(&self) -> HashMap<String, String> {
        self.args
            .iter()
            .map(|arg| (arg.key.clone(), arg.value.clone()))
            .collect()
    }
}

fn parse_key_value(s: &str) -> Result<KeyValueArg, String> {
    s.parse()
}

#[tokio::main]
async fn main() {
    let cli = Cli::from_args();

    for arg in &cli.args {
        println!("arg: {}={}", arg.key, arg.value);
    }

    let configuration = Configuration::new(&cli.api_key)
        .host(&cli.host)
        .user_agent(&cli.user_agent);
    cli.process(configuration)
        .await
        .expect("command processing failed");
}
