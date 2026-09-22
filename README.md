# decktags

Analyze a Magic: The Gathering decklist and discover which [Scryfall](https://scryfall.com) tags are most relevant to its cards.

`decktags` generates an overview of the Scryfall tags found across a deck, making it easier to identify useful tags when searching Scryfall for cards that fit the deck.

## Usage

```bash
decktags deck.txt
```

Example output:

```text
Scryfall tags
────────────────────────────
removal          ███████████████  15
draw             ███████████      11
ramp             █████████        9
token-maker      ███████          7
sacrifice        █████            5
...
```

Use the most prominent tags as starting points for further searches on Scryfall.

## License

See [LICENSE](LICENSE).
