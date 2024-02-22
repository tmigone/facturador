/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */

import Afip from '@afipsdk/afip.js'
import fs from 'fs'
import ora from 'ora'

import { AfipExtended } from '../types'
import { Command } from '@commander-js/extra-typings'

const program = new Command()
  .command('authorize-certificate')
  .description('Authorize certificate to use AFIP webservices')
  .summary('Authorize certificate')
  .requiredOption('-c, --cuit <cuit>', 'CUIT o CUIL del usuario')
  .requiredOption('-p, --password <password>', 'Clave fiscal del usuario')
  .action(async (args) => {
    const wsid = 'wsfe'
    const alias = 'afipsdk'
    const certFile = `cert/${alias}.crt`
    const keyFile = `cert/${alias}.key`

    const spinner = ora()

    // Check if cert files exist
    spinner.start('Checking if certificate files exist')
    if (!(fs.existsSync(certFile) && fs.existsSync(keyFile))) {
      spinner.fail('Certificate files not found, run generate-certificate first')
      return
    }
    spinner.succeed('Certificate files found')

    // Authorize certificate
    try {
      const afip = new Afip({ CUIT: args.cuit })
      spinner.start('Authorizing certificate')
      const res = await (afip as AfipExtended).CreateWSAuth(args.cuit, args.password, alias, wsid)
      spinner.succeed('Certificate authorized')
      console.log(res)
    }
    catch (error) {
      spinner.fail('Error authorizing certificate')
      console.log(error)
    }
  })

export default program
