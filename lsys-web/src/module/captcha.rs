use async_trait::async_trait;
use captcha::filters::{Dots, Noise, Wave};
use captcha::Captcha;
use lsys_core::{ValidCodeData, ValidCodeResult};
use redis::aio::Connection;

pub struct ValidCodeDataCaptcha {
    pub code: String,
    pub image_data: Vec<u8>,
    pub image_header: &'static str,
    pub save_time: usize,
}

#[async_trait]
impl ValidCodeData for ValidCodeDataCaptcha {
    async fn get_code<'t>(
        &mut self,
        _: &'t mut Connection,
        _: &'t Option<String>,
        _: &'t str,
        _: &'t str,
    ) -> ValidCodeResult<String> {
        Ok(self.code.to_owned())
    }
    async fn clear_code<'t>(
        &mut self,
        _: &'t mut Connection,
        _: &'t str,
        _: &'t str,
    ) -> ValidCodeResult<()> {
        Ok(())
    }
    fn save_time(&self) -> usize {
        self.save_time
    }
}

impl Default for ValidCodeDataCaptcha {
    fn default() -> Self {
        Self::new(120)
    }
}

impl ValidCodeDataCaptcha {
    pub fn new(save_time: usize) -> Self {
        let mut catpcha = Captcha::new();
        catpcha
            .add_chars(3)
            .apply_filter(Noise::new(0.2))
            .apply_filter(Wave::new(1.0, 5.0).horizontal())
            .apply_filter(Wave::new(1.0, 5.0).vertical())
            .view(150, 40)
            .apply_filter(Dots::new(3));
        Self {
            code: catpcha.chars_as_string(),
            image_header: "image/png",
            image_data: catpcha.as_png().unwrap_or_default(),
            save_time,
        }
    }
}
