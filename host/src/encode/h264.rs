#[derive(Debug, Clone, Copy)]
pub enum EncoderBackend {
    Vaapi,
    Software,
}

#[derive(Debug)]
pub struct H264Config {
    pub prefer_vaapi: bool,
}

#[derive(Debug)]
pub struct H264Encoder {
    backend: EncoderBackend,
}

pub fn init(config: H264Config) -> Result<H264Encoder, String> {
    if config.prefer_vaapi {
        match init_vaapi() {
            Ok(encoder) => return Ok(encoder),
            Err(error) => {
                println!("VAAPI unavailable, falling back to software: {error}");
            }
        }
    }

    init_software()
}

fn init_vaapi() -> Result<H264Encoder, String> {
    Err("VAAPI probing not implemented (stub)".to_string())
}

fn init_software() -> Result<H264Encoder, String> {
    println!("Configuring software H.264 encoder");
    Ok(H264Encoder {
        backend: EncoderBackend::Software,
    })
}
