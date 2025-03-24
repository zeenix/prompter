use std::collections::HashMap;

use zbus::{
    fdo, interface, proxy,
    zvariant::{OwnedObjectPath, OwnedValue, Value, SerializeDict, DeserializeDict, Type},
    Connection,
};

#[proxy(
    default_service = "org.gnome.keyring.SystemPrompter",
    interface = "org.gnome.keyring.internal.Prompter",
    default_path = "/org/gnome/keyring/Prompter"
)]
pub trait Prompter {
    fn begin_prompting(&self, callback: &OwnedObjectPath) -> Result<(), fdo::Error>;

    fn perform_prompt(
        &self,
        callback: OwnedObjectPath,
        type_: &str,
        properties: Properties,
        exchange: &str,
    ) -> Result<(), fdo::Error>;

    fn stop_prompting(&self, callback: OwnedObjectPath) -> Result<(), fdo::Error>;
}

#[derive(Debug, DeserializeDict, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct Properties {
    #[zvariant(rename = "continue-label")]
    continue_label: Option<String>,
    description: Option<String>,
    message: Option<String>,
    #[zvariant(rename = "caller-window")]
    caller_window: Option<String>,
    #[zvariant(rename = "cancel-label")]
    cancel_label: Option<String>,
}

pub struct PrompterCallback {
    path: OwnedObjectPath,
}

#[interface(name = "org.gnome.keyring.internal.Prompter.Callback")]
impl PrompterCallback {
    pub async fn prompt_ready(
        &self,
        reply: &str,
        _properties: Properties,
        exchange: &str,
        #[zbus(connection)] connection: &zbus::Connection,
    ) {
        if reply == "no" {
            std::process::exit(0);
        }

        let mut properties: HashMap<&str, OwnedValue> = HashMap::new();
        properties.insert("continue-label", Value::new("Lock").try_to_owned().unwrap());
        properties.insert(
            "description",
            Value::new(format!("Confirm locking '{}', Keyring.", "login"))
                .try_to_owned()
                .unwrap(),
        );
        properties.insert(
            "message",
            Value::new("Lock Keyring").try_to_owned().unwrap(),
        );
        properties.insert("caller-window", Value::new("").try_to_owned().unwrap());
        properties.insert("cancel-label", Value::new("Cancel").try_to_owned().unwrap());

        let p = Properties {
            continue_label: Some("Lock".to_owned()),
            description: Some("Confirm locking 'login' Keyring".to_owned()),
            message: Some("Lock Keyring".to_owned()),
            caller_window: Some(String::new()),
            cancel_label: Some("Cancel".to_owned()),
        };

        let path = self.path.clone();
        let connection = connection.clone();
        let exchange = exchange.to_owned();

        tokio::spawn(async move {
            let prompter = PrompterProxy::new(&connection).await.unwrap();
            prompter
                .perform_prompt(path, "confirm", p, &exchange)
                .await
                .unwrap();
        });
    }

    pub async fn prompt_done(&self) {}
}

impl PrompterCallback {
    pub async fn new() -> Self {
        Self {
            path: OwnedObjectPath::try_from("/org/gnome/keyring/Prompt/p6".to_string()).unwrap(),
        }
    }
}

#[tokio::main]
async fn main() {
    let connection = Connection::session().await.unwrap();

    let callback = PrompterCallback::new().await;
    connection
        .object_server()
        .at("/org/gnome/keyring/Prompt/p6", callback)
        .await
        .unwrap();

    let prompter = PrompterProxy::new(&connection).await.unwrap();
    let path = OwnedObjectPath::try_from("/org/gnome/keyring/Prompt/p6".to_string()).unwrap();
    prompter.begin_prompting(&path).await.unwrap();

    std::future::pending::<()>().await;
}
