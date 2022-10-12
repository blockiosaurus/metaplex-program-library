#!/usr/bin/env node

import chalk from 'chalk';
import clear from 'clear';
import * as figlet from 'figlet';
import * as path from 'path';
import { program } from 'commander';
import log from 'loglevel';
import * as sdk from '@metaplex-foundation/mpl-token-metadata/src/generated';
import * as web3 from '@solana/web3.js';
import * as fs from 'fs';
import { Keypair } from '@solana/web3.js';
import { keypairIdentity, Metaplex } from '@metaplex-foundation/js';
import { use_metaplex } from './helpers/utils';

clear();
console.log(chalk.green(figlet.textSync('Bundt Cake CLI', { horizontalLayout: 'full' })));

function programCommand(name: string) {
  return program
    .command(name)
    .option(
      '-e, --env <string>',
      'Solana cluster env name',
      'devnet', //mainnet-beta, testnet, devnet
    )
    .option('-r, --rpc <string>', 'The endpoint to connect to.')
    .option('-k, --keypair <path>', `Solana wallet location`, '--keypair not provided')
    .option('-l, --log-level <string>', 'log level', setLogLevel);
}

programCommand('create')
  .option('-a, --amount <string>', 'Amount to escrow')
  .action(async (directory, cmd) => {
    const { keypair, env, rpc } = cmd.opts();

    const metaplex = await use_metaplex(keypair, env, rpc);
  });

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function setLogLevel(value, prev) {
  if (value === undefined || value === null) {
    return;
  }
  log.info('setting the log value to: ' + value);
  log.setLevel(value);
}

program
  .version('0.0.1')
  .description('CLI for controlling the Bundt Cake feature on Token Metadata accounts.')
  .parse(process.argv);

console.log(sdk);
