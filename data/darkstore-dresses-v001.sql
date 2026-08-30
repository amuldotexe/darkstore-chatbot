-- Dark-store fashion concierge v001
-- Turso/libSQL-compatible seed dump.
-- Source-derived fields: sku through source_note.
-- Demo-fixture fields: fixture_available through fixture_style_tags_json.

BEGIN;

CREATE TABLE inventory_products (
  sku TEXT PRIMARY KEY,
  category_id TEXT NOT NULL CHECK (category_id = 'dresses'),
  brand TEXT NOT NULL,
  product_name TEXT NOT NULL,
  current_price_inr INTEGER NOT NULL CHECK (current_price_inr >= 0),
  mrp_inr INTEGER NOT NULL CHECK (mrp_inr >= current_price_inr),
  discount_percent INTEGER NOT NULL CHECK (discount_percent BETWEEN 0 AND 100),
  try_and_buy INTEGER NOT NULL CHECK (try_and_buy IN (0, 1)),
  merchandising_badge TEXT,
  source_catalog_url TEXT NOT NULL,
  source_product_url TEXT NOT NULL,
  source_captured_at_utc TEXT NOT NULL,
  source_listing_position INTEGER NOT NULL CHECK (source_listing_position >= 0),
  source_note TEXT NOT NULL,
  fixture_available INTEGER NOT NULL CHECK (fixture_available IN (0, 1)),
  fixture_sizes_json TEXT NOT NULL CHECK (json_array_length(fixture_sizes_json) BETWEEN 2 AND 3),
  fixture_delivery_minutes INTEGER NOT NULL CHECK (fixture_delivery_minutes > 0),
  fixture_propensity_score INTEGER NOT NULL CHECK (fixture_propensity_score BETWEEN 0 AND 100),
  fixture_dress_type TEXT NOT NULL,
  fixture_style_tags_json TEXT NOT NULL CHECK (json_array_length(fixture_style_tags_json) >= 1)
) STRICT;

INSERT INTO inventory_products (
  sku, category_id, brand, product_name, current_price_inr, mrp_inr,
  discount_percent, try_and_buy, merchandising_badge, source_catalog_url,
  source_product_url, source_captured_at_utc, source_listing_position, source_note,
  fixture_available, fixture_sizes_json, fixture_delivery_minutes,
  fixture_propensity_score, fixture_dress_type, fixture_style_tags_json
) VALUES
  (
    'SKID00083927', 'dresses', 'Slikk X Revolte',
    'Black Minimalist A-Line Evening Dress For Date Night', 1230, 2349,
    48, 1, NULL,
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/Slikk%20x%20Revolte/Black-Minimalist-A-Line-Evening-Dress-for-Date-Night/SKID00083927',
    '2026-08-29T05:49:09Z', 0,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["S","M","L"]', 50, 94, 'evening_dress', '["minimalist","date_night","a_line","black"]'
  ),
  (
    'SKID00167395', 'dresses', 'OUTZIDR',
    'Minimalist Beige Solid Straight-Fit Shirt Dress', 1367, 1599,
    15, 1, 'Crazy Deal',
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/OUTZIDR/Minimalist-Beige-Solid-Straight-Fit-Shirt-Dress/SKID00167395',
    '2026-08-29T05:49:09Z', 1,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["XS","S","M"]', 50, 80, 'shirt_dress', '["minimalist","everyday","straight_fit","beige"]'
  ),
  (
    'SKID00174036', 'dresses', 'MYWISHBAG',
    'Edgy Floral Ruched Black Party Bodycon Dress', 1300, 2999,
    57, 1, 'Crazy Deal',
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/MYWISHBAG/Edgy-Floral-Ruched-Black-Party-Bodycon-Dress/SKID00174036',
    '2026-08-29T05:49:09Z', 2,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["S","M","L"]', 50, 92, 'party_dress', '["party","bodycon","ruched","floral","black"]'
  ),
  (
    'SKID00081801', 'dresses', 'Slikk X Revolte',
    'Black Ruched Tube Dress For Date Nights', 432, 1999,
    78, 1, NULL,
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/Slikk%20x%20Revolte/Black-Ruched-Tube-Dress-for-Date-Nights/SKID00081801',
    '2026-08-29T05:49:09Z', 3,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["XS","S","M"]', 50, 89, 'tube_dress', '["date_night","ruched","tube","black"]'
  ),
  (
    'SKID00076560', 'dresses', 'OUTZIDR',
    'Light Blue Minimalist Belted Shirt Dress', 1237, 1649,
    25, 1, 'Crazy Deal',
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/OUTZIDR/Light-Blue-Minimalist-Belted-Shirt-Dress/SKID00076560',
    '2026-08-29T05:49:09Z', 4,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["S","M","L"]', 50, 80, 'shirt_dress', '["minimalist","belted","shirt_dress","light_blue"]'
  ),
  (
    'SKID00207435', 'dresses', 'BrownButter',
    'Coquette Hourglass A-Line Party Dress', 1499, 2000,
    25, 1, NULL,
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/BrownButter/Coquette-Hourglass-A-Line-Party-Dress/SKID00207435',
    '2026-08-29T05:49:09Z', 5,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["M","L"]', 50, 87, 'party_dress', '["coquette","party","hourglass","a_line"]'
  ),
  (
    'SKID00119053', 'dresses', 'Slikk X Revolte',
    'Minimalist Knotted Babydoll Black A-Line Dress', 711, 3000,
    76, 1, NULL,
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/Slikk%20x%20Revolte/Minimalist-Knotted-Babydoll-Black-A-Line-Dress/SKID00119053',
    '2026-08-29T05:49:09Z', 6,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["S","M"]', 50, 85, 'babydoll_dress', '["minimalist","babydoll","a_line","black"]'
  ),
  (
    'SKID00184392', 'dresses', 'Fiorra',
    'Orange Linen Minimalist A-Line Shirt Dress', 1619, 4599,
    65, 1, NULL,
    'https://www.slikk.club/products?banner_id=8084&namex=Dresses&section_name=Shop+By+Category+Diwali&section_id=8&banner_pos=8',
    'https://www.slikk.club/dresses/dresses/Fiorra/Orange-Linen-Minimalist-A-Line-Shirt-Dress/SKID00184392',
    '2026-08-29T05:49:09Z', 7,
    'Live DOM sample; price, availability, and delivery are point-in-time.',
    1, '["S","M","L"]', 50, 82, 'shirt_dress', '["linen","minimalist","a_line","orange"]'
  );

COMMIT;
