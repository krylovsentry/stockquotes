# stockquotes

[![CI](https://github.com/krylovsentry/stockquotes/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/krylovsentry/stockquotes/actions/workflows/rust-ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/krylovsentry/stockquotes/badge.svg?branch=main)](https://coveralls.io/github/krylovsentry/stockquotes?branch=main)

Workspace с двумя приложениями для потоковой передачи котировок акций по сети: **сервер** генерирует данные и рассылает их по UDP, **клиент** подключается по TCP, отправляет команду подписки и получает котировки по UDP.

## Структура проекта

| Крейт           | Описание |
|-----------------|----------|
| `quote_lib`     | Общая библиотека: модель `StockQuote`, сериализация в bencode (bendy). |
| `quote_server`  | TCP/UDP-сервер: генератор котировок, диспетчер по подписчикам, разбор команды `STREAM`, Ping/Keep-Alive. |
| `quote_client`  | Клиент: CLI (clap), загрузка тикеров из файла, отправка `STREAM` по TCP, приём котировок по UDP, поток Ping, Ctrl+C. |

## Требования

- Файл с тикерами (по одному на строку), например `tickers.txt` в корне или в `quote_server/stock_data/tickers.txt` для сервера.


## Запуск

### 1. Сервер

Сервер слушает TCP на `127.0.0.1:34254`. Тикеры загружаются из `quote_server/stock_data/tickers.txt`.

```bash
cargo run -p quote_server
```

Убедитесь, что файл тикеров существует (например, создайте `quote_server/stock_data/tickers.txt` с содержимым:

```
AAPL
GOOGL
TSLA
```

### 2. Файл тикеров для клиента

Создайте файл с тикерами, которые клиент будет запрашивать (по одному на строку), например в корне проекта `tickers.txt`:

```
AAPL
TSLA
```

### 3. Клиент

Клиент подключается к серверу по TCP, отправляет команду `STREAM`, затем принимает котировки по UDP на указанный порт. Раз в 2 секунды шлёт Ping на сервер. Завершение по Ctrl+C.

```bash
cargo run -p quote_client -- --server-addr 127.0.0.1:34254 --udp-port 40000 --tickers-file tickers.txt
```

Параметры:

- `--server-addr` — адрес и порт TCP-сервера (как в примере выше).
- `--udp-port` — порт на стороне клиента для приёма UDP-котировок (должен совпадать с портом в команде STREAM; сервер шлёт на `127.0.0.1:40000` при таком вызове).
- `--tickers-file` — путь к файлу с тикерами.

### Пример полного запуска (два терминала)

**Терминал 1 — сервер:**

```bash
cargo run -p quote_server
```

**Терминал 2 — клиент:**

```bash
cargo run -p quote_client -- --server-addr 127.0.0.1:34254 --udp-port 40000 --tickers-file tickers.txt
```

В терминале клиента появятся строки вида:

```
Sent STREAM command for 2 ticker(s) to 127.0.0.1:34254
AAPL price=... volume=... ts=...
TSLA price=... volume=...
...
```

Остановка клиента: Ctrl+C (выведется "Shutting down.").

## Тесты

```bash
cargo test --workspace
```

## Линтинг

```bash
cargo clippy --workspace --all-targets -- -D warnings
```
