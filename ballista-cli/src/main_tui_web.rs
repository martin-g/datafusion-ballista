// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

mod tui;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
use tokio::runtime::Builder;

#[derive(Debug)]
struct Args {
    // #[clap(long, help = "Ballista scheduler host")]
    host: Option<String>,

    // #[clap(long, help = "Ballista scheduler port")]
    port: Option<u16>,

    // #[clap(
    //     short,
    //     long,
    //     help = "Reduce printing other than the results and work quietly"
    // )]
    quiet: bool,
}

// #[tokio::main(flavor = "current_thread")]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let args = Args {
                host: Some("localhost".to_string()),
                port: Some(50050),
                quiet: false,
            };

            tui::tui_main()
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
}
