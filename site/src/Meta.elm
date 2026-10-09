module Meta exposing (Group, Meta, decoder, groups, piecesDecoder)

{-| What a piece says about itself, as `Meta::to_json` in
crates/animate-unicode/src/meta.rs writes it, and the sidebar's grouping.
-}

import Json.Decode as D


{-| One piece's meta. `options` is the JSON object of option defaults, as text.
-}
type alias Meta =
    { name : String
    , slug : String
    , category : String
    , note : String
    , cols : Int
    , rows : Int
    , fps : Int
    , charset : String
    , options : Maybe String
    , clock : Bool
    , palette : Maybe (List String)
    , ground : Maybe String
    , cell : Int
    }


decoder : D.Decoder Meta
decoder =
    D.succeed Meta
        |> field "name" D.string
        |> field "slug" D.string
        |> field "category" D.string
        |> field "note" D.string
        |> field "cols" D.int
        |> field "rows" D.int
        |> field "fps" D.int
        |> field "charset" D.string
        |> field "options" (D.nullable D.string)
        |> field "clock" D.bool
        |> field "palette" (D.nullable (D.list D.string))
        |> field "ground" (D.nullable D.string)
        |> field "cell" D.int


field : String -> D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
field name fieldDecoder =
    D.map2 (|>) (D.field name fieldDecoder)


{-| The flags main.ts passes: `{ pieces: [Meta] }`.
-}
piecesDecoder : D.Decoder (List Meta)
piecesDecoder =
    D.field "pieces" (D.list decoder)


{-| A sidebar group: a label and its categories, in order.
-}
type alias Group =
    { label : String
    , categories : List String
    }


{-| Sidebar order, after ascii.rest: the scenes first, then the pieces meant as
page furniture, then the rest. Every `Category` in meta.rs is in exactly one.
-}
groups : List Group
groups =
    [ { label = "art", categories = [ "scenes" ] }
    , { label = "components", categories = [ "ui", "data", "type", "logos" ] }
    , { label = "more", categories = [ "shapes", "space", "physics", "nature", "creatures", "objects", "generative", "effects" ] }
    ]
