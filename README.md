# Facturador AFIP

## Descripción

Este proyecto es un facturador para la AFIP. Permite generar facturas electrónicas desde la consola.

## Instalación

```bash
yarn install
```

## Uso

### Configuracion
1. Generar un certificado y una clave privada. Se puede hacer con el siguiente comando:

```bash
ts-node src/index.ts generate-certificate --cuit <cuit/cuil> --password <clave-fiscal-afip>
```