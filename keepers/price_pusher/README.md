# Price Pusher

Find the implementation of the price pusher at https://github.com/pyth-network/pyth-crosschain/tree/main/apps/price_pusher.

## Configure environment variables

```bash
cp .env.example .env
```

Edit the `.env` file with the correct values.

## Run the price pusher

```bash
docker compose up -d
```

### Stop the price pusher

```bash
docker compose down
```

### Stream logs

```bash
docker compose logs -f
```
