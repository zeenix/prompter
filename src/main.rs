use std::collections::HashMap;

use zbus::{
    fdo, interface, proxy,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
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
        properties: HashMap<&str, OwnedValue>,
        exchange: &str,
    ) -> Result<(), fdo::Error>;

    fn stop_prompting(&self, callback: OwnedObjectPath) -> Result<(), fdo::Error>;
}

pub struct PrompterCallback {
    path: OwnedObjectPath,
}

#[interface(name = "org.gnome.keyring.internal.Prompter.Callback")]
impl PrompterCallback {
    pub async fn prompt_ready(
        &self,
        _reply: &str,
        _properties: HashMap<&str, OwnedValue>,
        exchange: &str,
        #[zbus(connection)] connection: &zbus::Connection,
    ) {
        println!("{}", exchange);

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

        let prompter = PrompterProxy::new(&connection).await.unwrap();
        prompter
            .perform_prompt(self.path.clone(), "confirm", properties, exchange)
            .await
            .unwrap();
    }

    pub async fn prompt_done(&self) {}
}

impl PrompterCallback {
    pub async fn new() -> Self {
        Self {
            path: OwnedObjectPath::try_from(format!("/org/gnome/keyring/Prompt/p6")).unwrap(),
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
    let path = OwnedObjectPath::try_from(format!("/org/gnome/keyring/Prompt/p6")).unwrap();
    prompter.begin_prompting(&path).await.unwrap();

    std::future::pending::<()>().await;
}
