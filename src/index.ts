import { program } from '@commander-js/extra-typings'

program.command('print')
  .argument('<file>')
  .option('--double-sided')
  .action((args) => {
    console.log(args)
  })

program.parse()
