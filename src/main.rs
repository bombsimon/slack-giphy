use slack::api::MessageStandard;
use slack::{Event, EventHandler, Message, RtmClient};

struct GiphyBot {
    /// GiphyBot is the bot that sends gifs to Slack based on configured keywords.
    giphy_api_key: String,
    giphy_keywords: Vec<String>,
}

#[allow(unused_variables)]
impl EventHandler for GiphyBot {
    /// Handles every `slack::Event` and takes appropreate action for each event.
    ///
    /// # Arguments
    ///
    /// * `cli` - The Slack RtmClient
    ///
    /// * `event` - The event from Slack
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        match event {
            Event::Message(msg) => {
                let (text, channel_id) = match *msg {
                    Message::Standard(MessageStandard {
                        ref text,
                        ref channel,
                        ..
                    }) => (text.as_ref().unwrap(), channel.as_ref().unwrap()),
                    _ => {
                        println!("could not get text and channel");
                        return;
                    }
                };

                for tag in &self.giphy_keywords {
                    if text.to_lowercase().contains(tag) {
                        match get_giph(&self.giphy_api_key, tag) {
                            Some(giph) => {
                                let _ = cli.sender().send_message(&channel_id, giph.as_str());
                                break;
                            }
                            None => println!("could not get gif"),
                        }
                    }
                }
            }
            _ => println!("on_event(event: {:?})", event),
        };
    }

    /// Called when the Slack connection is shutting down.
    ///
    /// # Arguments
    ///
    /// * `cli` - The Slack RtmClient
    fn on_close(&mut self, cli: &RtmClient) {
        println!("on_close");
    }

    /// Called when the Slack connection is established.
    ///
    /// # Arguments
    ///
    /// * `cli` - The Slack RtmClient
    fn on_connect(&mut self, cli: &RtmClient) {
        println!("connected");
    }
}

/// Returns an Option with Some(gif url) or None if an HTTP error occurs.
///
/// # Arguments
///
/// * `api_key` - The Giphy.com API key.
///
/// * `tag` - The tag to search for.
fn get_giph(api_key: &str, tag: &str) -> Option<String> {
    let request_url = format!(
        "http://api.giphy.com/v1/gifs/random?api_key={api_key}&tag={tag}",
        api_key = api_key,
        tag = tag
    );

    let result: Result<serde_yaml::Value, reqwest::Error> =
        reqwest::get(&request_url).and_then(|mut response| response.json());

    match result {
        Ok(v) => v["data"]["url"].as_str().and_then(|u| Some(u.to_string())),
        _ => None,
    }
}

/// The main method will setup the Slack RtmClient and start listening on events.
fn main() {
    let f = std::fs::File::open("config.yaml").unwrap();
    let config: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();
    let mut tags: Vec<String> = vec![];

    for tag in config["giphy_tags"].as_sequence().unwrap() {
        tags.push(String::from(tag.as_str().unwrap()));
    }

    let mut handler = GiphyBot {
        giphy_api_key: String::from(config["giphy"].as_str().unwrap()),
        giphy_keywords: tags,
    };

    let r = RtmClient::login_and_run(config["slack"].as_str().unwrap(), &mut handler);

    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
