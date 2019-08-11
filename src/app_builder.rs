use crate::app::App;
use crate::exposed_tools::*;

pub struct AppBuilder {
    app: App,
}

impl AppBuilder {
    pub fn default() -> Self {
        let app = App::new();

        AppBuilder { app: app }
    }

    pub fn with_multisampling(mut self) -> Self {
        self.app.enable_multisampling();
        self
    }

    pub fn with_camera(mut self, camera: Box<dyn Camera>) -> Self {
        self.app.update_camera(camera);
        self
    }

    pub fn build(self) -> App {
        self.app
    }
}
