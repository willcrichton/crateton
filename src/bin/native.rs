use rg3d::core::futures::executor;

fn main() {
  executor::block_on(async {
    crateton::run().await;
  });
}
