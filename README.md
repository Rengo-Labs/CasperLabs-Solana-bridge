# CasperLabs-Solana-bridge

## Building
### Prerequisits
Each list item is linked to it's installation page.
- [Rust](https://www.rust-lang.org/tools/install)
- [NodeJS](https://heynode.com/tutorial/install-nodejs-locally-nvm/) using NVM, version 14.0.0 or more.
- [Solana CLI Toolsuite](https://docs.solana.com/cli/install-solana-cli-tools)
- [solana-test-validator](https://docs.solana.com/developing/test-validator)
### Build
- Install npm dependencies with `npm i`.

- Ensure you have the stable RUST toolchain with `rustup default stable`. BPF toolchain will be installed by cargo as a dependency of the project.

- The program can be build with `npm run build:program`. 
This will output the `.so` shared object file and the program ids to `target/bpfel-unknown-unknown/release` folder.

## Testing
- Host based and `solana-test-validator` based testing can be triggered with `npm test`, this will spin up `solana-test-validator` itself.

- Host and `solana-test-validator` test can be done individually by `npm run test-host` and `npm run test-client` respectfully.

## Deployment
Deployment payer, target network, and the program id's of the deployment are subject to the configuration of your solana CLI.
`solana config get` will display your current configuration.
Detailed steps to configure said CLI are provided by Solana [here](https://docs.solana.com/cli/deploy-a-program).

Once your Solana CLI config is set as desired, `npm run deploy` will deploy all programs in this repository.
