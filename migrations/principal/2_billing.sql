CREATE TABLE IF NOT EXISTS products (
    pk bigserial PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    quota BIGINT NOT NULL CHECK (quota >= 0),
    deactivated_at TIMESTAMP,
    slug VARCHAR(255) NOT NULL UNIQUE,
    product_stripe_id VARCHAR(255) NOT NULL UNIQUE,
    price_stripe_id VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS customers (
    pk bigserial PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    stripe_id VARCHAR(255) NOT NULL UNIQUE,
    user_pk BIGINT NOT NULL UNIQUE,
    CONSTRAINT fk_user FOREIGN KEY (user_pk) REFERENCES users (pk) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS purchases (
    pk bigserial PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    invoice_id VARCHAR(255) NOT NULL UNIQUE,
    customer_pk BIGINT NOT NULL,
    product_pk BIGINT NOT NULL,
    CONSTRAINT fk_customer FOREIGN KEY (customer_pk) REFERENCES customers (pk) ON DELETE CASCADE,
    CONSTRAINT fk_product FOREIGN KEY (product_pk) REFERENCES products (pk) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_products_stripe_id ON products(product_stripe_id);
CREATE UNIQUE INDEX idx_customers_stripe_id ON customers(stripe_id);
