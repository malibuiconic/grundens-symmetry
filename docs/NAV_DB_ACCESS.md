# Grundens NAV18 Database Access

## Overview

This middleware connects directly to Grundens' Microsoft Dynamics NAV 18 (and Integration) databases hosted on Azure SQL. The connection requires Cisco Secure Connect VPN access to the Grundens corporate network.

---

## Architecture

```
malibudev (Rust app)
    └── localhost:1433
            └── SSH tunnel → Mac (192.168.0.248)
                    └── Cisco Secure Connect VPN
                            └── sql-grus-prd-01.database.windows.net:1433
                                    ├── sqldb-nav18-grus-prd-01       (NAV18 production)
                                    └── sqldb-grusintegration-prd-01  (Integration DB)
```

Both databases live on the same Azure SQL Server instance. The SSH tunnel through a VPN-connected Mac is the access path for the `malibudev` remote server.

---

## Prerequisites

1. **Cisco Secure Connect** installed and connected on your Mac (`vpn.grundens.com`)
   - Authenticate with your Grundens SSO email/password
2. **Remote Login (SSH) enabled** on your Mac
   - System Settings → General → Sharing → Remote Login → ON
3. **SSH tunnel running** from `malibudev` to your Mac

---

## Starting the SSH Tunnel

Run this on `malibudev` each time before starting the application:

```bash
ssh -N -L 1433:sql-grus-prd-01.database.windows.net:1433 malibu@192.168.0.248
```

Leave this terminal open — the `-N` flag keeps it alive with no output. The tunnel forwards `localhost:1433` on `malibudev` through your Mac's VPN connection to the Azure SQL server.

> Replace `192.168.0.248` with your Mac's current local IP if it changes (`ipconfig getifaddr en0`).

---

## Environment Variables

Configured in `.env` at the project root:

| Variable | Value | Notes |
|---|---|---|
| `NAV_SQL_HOST` | `localhost` | Points to SSH tunnel |
| `NAV_SQL_PORT` | `1433` | Standard SQL Server port |
| `NAV_SQL_USER` | `GrusAdmin@sql-grus-prd-01` | Azure SQL requires `user@server` format |
| `NAV_SQL_PASSWORD` | *(see .env)* | |
| `NAV_SQL_DB_NAV18` | `sqldb-nav18-grus-prd-01` | NAV18 production database |
| `NAV_SQL_DB_INTEGRATION` | `sqldb-grusintegration-prd-01` | Integration database |

> **Important:** Azure SQL requires the username in `username@servername` format when the connection hostname differs from the actual server (e.g. when tunneling through localhost).

---

## Startup Verification

When the app starts, it automatically probes both DB connections and logs the result:

```
INFO  shopify_app::nav > NAV18 DB connected (sqldb-nav18-grus-prd-01)
INFO  shopify_app::nav > Integration DB connected (sqldb-grusintegration-prd-01)
```

If the tunnel is down or VPN is disconnected, you'll see a connection error instead and the app will still start — DB endpoints will return errors until the tunnel is restored.

---

## API Endpoints

All endpoints are read-only against NAV18.

### `GET /api/nav/status`
Tests live connectivity to both databases. Returns connection status and SQL Server version.

```bash
curl http://localhost:3030/api/nav/status
```

```json
{
  "nav18": {
    "database": "sqldb-nav18-grus-prd-01",
    "connected": true,
    "message": "Connection successful",
    "server_version": "Microsoft SQL Azure (RTM) - 12.0.2000.8 ..."
  },
  "integration": { ... }
}
```

---

### `GET /api/nav/items`
Returns the first 20 active items from `[GRUS$Item]` — SKU, description, and unit price.

```bash
curl http://localhost:3030/api/nav/items
```

```json
[
  { "sku": "10000-800-0020", "description": "Balder Jacket 302", "unit_price": 0.0 },
  ...
]
```

---

### `GET /api/nav/inventory`
Returns on-hand quantity per SKU at the TAC (US domestic) warehouse location from `[GRUS$Item Ledger Entry]`.

```bash
curl http://localhost:3030/api/nav/inventory
```

```json
[
  { "sku": "10000-800-0020", "qty_on_hand": 13 },
  ...
]
```

---

## Troubleshooting

| Error | Cause | Fix |
|---|---|---|
| `Could not read provided CA certificate` | `trust_cert_ca(false)` bug (fixed) | Resolved — uses `trust_cert()` now |
| `Connection reset by peer` | Tunnel active but wrong server hostname | Ensure tunnel points to `sql-grus-prd-01.database.windows.net` |
| `nodename nor servname provided` | Mac not on VPN | Connect Cisco Secure Connect first |
| `Cannot open server 'localhost'` | Missing `@servername` in username | Username must be `GrusAdmin@sql-grus-prd-01` |
| `Connection refused` on port 1433 | SSH tunnel not running | Re-run the tunnel command above |
| `Connection timed out` | Mac VPN disconnected | Reconnect Cisco Secure Connect on Mac |

---

## Reference

- Azure SQL Server: `sql-grus-prd-01.database.windows.net`
- VPN Gateway: `vpn.grundens.com`
- Credentials reference: `Dickens:BartAPI:Reference/Connor/source/Connor.Console/appsettings.json`
- Tiberius (Rust MSSQL driver): [docs.rs/tiberius](https://docs.rs/tiberius)
