mod helpers;
use std::error::Error;
pub use helpers::split_image;

pub struct Config {
    pub image_file: String,
    pub m_primary: (u32,u32),
    pub m_secondary: (u32,u32),
    pub offset: u32,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &str> {
        // handle args
        if args.len() < 5 {
            return Err("not enough arguments");
        } else if args.len() > 6 {
            return Err("too many arguments");
        } else if args[4].parse::<u32>().is_err() {
            return Err("offset is non-int");
        }

        // parse monitor vectors
        let m_primary = helpers::parse_resolution(args[2].clone())
            .map_err(
                |_| "Invalid Primary Resolution"
            )?;
        let m_secondary = helpers::parse_resolution(args[3].clone())
            .map_err(
                |_| "Invalid Secondary Resolution"
            )?;

        // clone args to vars
        let image_file: String = args[1].clone();
        let offset: u32 = args[4].clone()
                                .trim()
                                .parse()
                                .unwrap();

        // pass config
        Ok(Config {
            image_file,
            m_primary,
            m_secondary,
            offset
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // perform operation
    let images = split_image(
        &config.image_file,
        &config.m_primary,
        &config.m_secondary,
        config.offset
    ).map_err(
       |_| "Split Error"
    )?;

    // export images
    for image in images {
        let image_name = format!(
            "out_img_{}x{}_{}.png",
            image.width(),
            image.height(),
            &config.offset
        );
        image.save(
            image_name
        ).map_err(
            |_| "Split Error"
        )?;
    }

    // return none if success
    Ok(())
}
