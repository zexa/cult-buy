# cult-buy
Cult.buy is a Drop.com alternative. For profit, but part of the proceeds go to open source communities.

## 2021-06-27
Goals:
* Logging
* Image validation
* Image Struct
* Maybe database?

Achieved:
* Logging
* Image validation
* Image Struct
* (Partial) User Struct

What's left:
* image_validor should check that Content-Type correlates with image
* Files/Images are still pending a database
* Files should be moved into a proper directory.

## Schema
```
Users {
  id, // backendonly
  hash,
  email,
  login_codes: {},
  sessions: {},
  cart: Cart,
  created_at: DateTime,
  modified_at: DateTime,
  shipping_address: ShippingAddress,
  has_signed_up_for_newsletter: bool,
}

// Redis
LoginCodes {
  id,
  hash,
  secret,
  timeout: DateTime, // 15 minutes
  created_at: DateTime,
}

// Redis
Session {
  id,
  hash,
  secret,
  timeout: DateTime,
  created_at: DateTime,
}

Listings {
  id, // backened only
  hash,
  name,
  price: Money,
  image: Image,
  created_at: DateTime,
  modified_at: DateTime,
}

Cart {
  hash,
  cart_items: Vec<CartItems>
  created_at: DateTime,
  modified_at: DateTime,
  Payment: Optional<Payment>,
}

CartItem {
  listing: CartItem,
  amount: Money,
  created_at: DateTime,
  modified_at: DateTime,
}

Money {} // from a lib

enum ImageAssignables {
  Listing,
  User,
}

Image {
  hash,
  file: File,
  created_at: DateTime,
  modified_at: DateTime,
  assigned_to: ImageAssignables, // Listing, 
  assigned_at: DateTime,
}

File {
  id,
  hash,
  source, // AWS S3, FTP,
  link,
  created_at: DateTime,
  assigned_to: String, // Image,
  assigned_at: DateTime,
}

ShippingAddress {
  hash,
  country,
  state,
  adress,
  postal_code,
  phone_number,  
}

Payment {} // Skrill

Delivery {
  hash,
  shipping_address: ShippingAddress,
  history, // Shipping to ShippingAddress, DeliveryAtCountryOfOrigin, DeliveryAtCountryDestination
  status: DeliveryStatus, // NotYetShipped,DeliveryAtCountryOfOrigin, ...
}
```

