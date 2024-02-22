import { Command } from '@commander-js/extra-typings'

import authorizeCertificate from './commands/authorize-certificate'
import generateCertificate from './commands/generate-certificate'

const program = new Command()
  .name('afip')
  .description('CLI tool for AFIP SDK')
program.addCommand(authorizeCertificate)
program.addCommand(generateCertificate)

program.parse()
