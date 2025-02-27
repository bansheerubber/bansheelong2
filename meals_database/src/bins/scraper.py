import json
import sys
from recipe_scrapers import scrape_me

url = sys.argv[1]
scraper = scrape_me(url)
print(json.dumps(scraper.to_json()))
