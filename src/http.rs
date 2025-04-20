use std::sync::OnceLock;

use anyhow::Result;
use reqwest::{
    IntoUrl,
    blocking::{Client, Response},
};

static CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> Result<&'static Client> {
    Ok(match CLIENT.get() {
        Some(client) => client,
        None => {
            let client = Client::builder().build()?;
            CLIENT.get_or_init(|| client)
        }
    })
}

pub fn get<U>(url: U) -> Result<Response>
where
    U: IntoUrl,
{
    let client = client()?;
    let request = client.get(url).build()?;
    Ok(client.execute(request)?)
}
