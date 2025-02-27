import json
import sys
import urllib.request
from urllib.request import urlopen
from recipe_scrapers import AbstractScraper, scrape_html

def scrape_me(url: str) -> AbstractScraper:
	req = urllib.request.Request(
		url,
		data=None,
		headers={
			'User-Agent': ''
		}
	)

	html = urlopen(req).read().decode("utf-8")
	return scrape_html(html, org_url=url)

url = sys.argv[1]
scraper = scrape_me(url)
print(json.dumps(scraper.to_json()))
