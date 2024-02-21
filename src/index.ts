import { Command } from '@commander-js/extra-typings'

import generateCertificate from './commands/generate-certificate'

const program = new Command()
  .name('afip')
  .description('CLI tool for AFIP SDK')
program.addCommand(generateCertificate)

program.parse()
