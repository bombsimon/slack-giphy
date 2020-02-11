use slack::api::MessageStandard;
use slack::{Event, EventHandler, Message, RtmClient};

struct GiphyBot {
    channel_name: String,
    channel_id: String,
    giphy_api_key: String,
    giphy_keywords: Vec<String>,
}

#[allow(unused_variables)]
impl EventHandler for GiphyBot {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        match event {
            Event::Hello => {
                let _ = cli
                    .sender()
                    .send_message(&self.channel_id, "Hello world! (rtm)");
            }
            Event::Message(msg) => {
                let text = match *msg {
                    Message::Standard(MessageStandard { ref text, .. }) => text.as_ref().unwrap(),
                    _ => "",
                };

                for tag in &self.giphy_keywords {
                    if text.to_lowercase().contains(tag) {
                        match get_giph(&self.giphy_api_key, tag) {
                            Some(giph) => {
                                let _ = cli.sender().send_message(&self.channel_id, giph.as_str());
                            }
                            None => println!("could not get giph"),
                        }
                    }
                }
            }
            _ => println!("on_event(event: {:?})", event),
        };
    }

    fn on_close(&mut self, cli: &RtmClient) {
        println!("on_close");
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        let channel_id = cli
            .start_response()
            //.channels // Public chats, does not list private ones
            .groups // Private chats, does not list public ones
            .as_ref()
            .and_then(|groups| {
                groups.iter().find(|group| match group.name {
                    None => false,
                    Some(ref name) => *name == self.channel_name,
                })
            })
            .and_then(|group| group.id.as_ref())
            .expect("group not found");

        self.channel_id = channel_id.to_string();
    }
}

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

fn main() {
    let f = std::fs::File::open("config.yaml").unwrap();
    let config: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();
    let mut tags: Vec<String> = vec![];

    for tag in config["giphy_tags"].as_sequence().unwrap() {
        tags.push(String::from(tag.as_str().unwrap()));
    }

    let mut handler = GiphyBot {
        channel_name: String::from(config["slack-channel"].as_str().unwrap()),
        channel_id: String::from(""),
        giphy_api_key: String::from(config["giphy"].as_str().unwrap()),
        giphy_keywords: tags,
    };

    let r = RtmClient::login_and_run(config["slack"].as_str().unwrap(), &mut handler);

    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
