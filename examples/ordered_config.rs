use components::*;
use serde::de::{self, Deserialize, Visitor};

#[derive(Debug, serde::Deserialize)]
struct OrderedComponent<T> {
    position: u8,
    component: T,
}

#[derive(Debug)]
struct OrderedConfig {
    banner: Option<OrderedComponent<BannerCfg>>,
    docker: Option<OrderedComponent<DockerConfig>>,
    last_login: Option<OrderedComponent<LastLoginCfg>>,
    last_run: Option<OrderedComponent<LastRunConfig>>,
}

// https://serde.rs/deserialize-struct.html
impl<'de> Deserialize<'de> for OrderedConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {

        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Banner,
            Docker,
            LastLogin,
            LastRun
        }

        struct ConfigVisitor;

        impl<'de> Visitor<'de> for ConfigVisitor {
            type Value = OrderedConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct OrderedConfig")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                Err(de::Error::custom("Arthhhh"))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>, {
                        let mut banner = None;
                        let mut docker = None;
                        let mut last_login = None;
                        let mut last_run = None;
                        let mut position = 0;

                        while let Some(key) = map.next_key()? {
                            match key {
                                Field::Banner => {
                                    println!("{} = banner", position);
                                    banner = Some(OrderedComponent {
                                        position,
                                        component: map.next_value::<BannerCfg>()?,
                                    });
                                }
                                Field::Docker => {
                                    println!("{} = docker", position);
                                    docker = Some(OrderedComponent {
                                        position,
                                        component: map.next_value::<DockerConfig>()?,
                                    });
                                }
                                Field::LastLogin => {
                                    println!("{} = last_login", position);
                                    last_login = Some(OrderedComponent {
                                        position,
                                        component: map.next_value::<LastLoginCfg>()?,
                                    });
                                }
                                Field::LastRun => {
                                    println!("{} = last_run", position);
                                    last_run = Some(OrderedComponent {
                                        position,
                                        component: map.next_value::<LastRunConfig>()?,
                                    });
                                }
                            }
                            position += 1;
                        }
                        Ok(OrderedConfig {
                            banner,
                            docker,
                            last_login,
                            last_run,
                        })
            }

        }


        const _FIELDS: &[&str] = &[
            "banner",
            "docker",
            "last_login",
            "last_run",
        ];
        // deserializer.deserialize_struct("OrderedConfig", FIELDS, ConfigVisitor)
        deserializer.deserialize_map(ConfigVisitor)
    }
}



fn main() {
    let config = "
[last_run]

[last_login]
user = 1
user2 = 2

[banner]
color = \"white\"
command = \"date\"

[docker]
\"/blah\" = \"blah\"
    ";

    let x: OrderedConfig = toml::from_str::<OrderedConfig>(config).unwrap();
    if let Some(last_login) = x.last_login {
        println!("last_login as position {}", last_login.position);
        for u in last_login.component.iter() {
            println!("    user: {}", u.0)
        }
    }
    todo!("implement iterator sorting by position");


}

mod components {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct BannerCfg {
        color: BannerColor,
        command: String,
    }

    #[derive(Debug, Deserialize)]
    enum BannerColor {
        #[serde(alias = "black")]
        Black,
        #[serde(alias = "red")]
        Red,
        #[serde(alias = "green")]
        Green,
        #[serde(alias = "yellow")]
        Yellow,
        #[serde(alias = "blue")]
        Blue,
        #[serde(alias = "magenta")]
        Magenta,
        #[serde(alias = "cyan")]
        Cyan,
        #[serde(alias = "white")]
        White,
        #[serde(alias = "light_black")]
        LightBlack,
        #[serde(alias = "light_red")]
        LightRed,
        #[serde(alias = "light_green")]
        LightGreen,
        #[serde(alias = "light_yellow")]
        LightYellow,
        #[serde(alias = "light_blue")]
        LightBlue,
        #[serde(alias = "light_magenta")]
        LightMagenta,
        #[serde(alias = "light_cyan")]
        LightCyan,
        #[serde(alias = "light_white")]
        LightWhite,
    }

    pub type DockerConfig = HashMap<String, String>;
    pub type LastLoginCfg = HashMap<String, usize>;

    #[derive(Debug, Deserialize)]
    pub struct LastRunConfig {}

}
