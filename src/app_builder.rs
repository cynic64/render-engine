use crate::app::App;

pub struct AppBuilder {
    app: Option<App>,
}

impl AppBuilder {
    pub fn default() -> Self {
        let app = App::new();

        AppBuilder {
            app: Some(app),
        }
    }

    pub fn build(&mut self) -> App {
        self.app.take().unwrap()
    }
}
