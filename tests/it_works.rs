use std::time::Duration;

use timer::TimerFuture;

use executor::new_executor_and_spawner;

#[test]
fn it_works() {
    let (executor, spawner) = new_executor_and_spawner();
    spawner.spawn(async {
        println!("howdy!");
        TimerFuture::new(Duration::new(4, 0)).await;
        println!("done!")
    });

    drop(spawner);
    executor.run()
}