use openfab_bambu::BambuDriver;
use openfab_drivers::DeviceDriver;
use openfab_drivers::events::EventType;
use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverRuntime;
use openfab_prusa::PrusaDriver;

#[tokio::main]
async fn main() {
    let mut runtime = NativeRuntime::new();
    let mut bambu = BambuDriver::new();
    let mut prusa = PrusaDriver::new();
    runtime.load(bambu).await.unwrap();
    runtime.load(prusa).await.unwrap();

    let drivers = runtime.index_drivers().await.unwrap();
    /*
    siv.add_layer(
        Dialog::new()
            .title("Printer Configuration")
            .content(
                ListView::new()
                    .child("IP Address:", EditView::new().with_name("ip"))
                    .child("API Key:", EditView::new().secret().with_name("key"))
                    .child("Enable TLS:", Checkbox::new()),
            )
            .button("Save", |s| s.quit()),
    );

    let mut siv = cursive::default();*/
}
