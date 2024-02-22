/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */

import Afip from '@afipsdk/afip.js'
import fs from 'fs'
import ora from 'ora'

import { AfipExtended } from '../types'
import { Command } from '@commander-js/extra-typings'
import { confirm } from '@inquirer/prompts'

const program = new Command()
  .command('generate-certificate')
  .description('Generate certificate to use AFIP webservices')
  .summary('Generate certificate')
  .requiredOption('-c, --cuit <cuit>', 'CUIT o CUIL del usuario')
  .requiredOption('-p, --password <password>', 'Clave fiscal del usuario')
  .action(async (args) => {
    const alias = 'afipsdk'
    const certFile = `cert/${alias}.crt`
    const keyFile = `cert/${alias}.key`

    const spinner = ora()

    // Check if cert files exist
    if (fs.existsSync(certFile) && fs.existsSync(keyFile)) {
      const answer = await confirm({ message: 'Cert file already exists. Do you want to overwrite it?' })
      if (!answer) {
        return
      }
    }
    else if (!fs.existsSync('cert')) {
      fs.mkdirSync('cert')
    }

    // Get cert from AFIP
    try {
      const afip = new Afip({ CUIT: args.cuit })
      spinner.start('Getting certificate from AFIP')
      const cert = await (afip as AfipExtended).CreateCert(args.cuit, args.password, alias)
      spinner.succeed('Certificate obtained')

      // Save cert to file
      spinner.start('Saving certificate to file')
      fs.writeFileSync(certFile, cert.cert)
      fs.writeFileSync(keyFile, cert.key)
      spinner.succeed('Certificate saved')
    }
    catch (error) {
      spinner.fail('Error getting certificate from AFIP')
      console.log(error)
    }
  })

export default program
