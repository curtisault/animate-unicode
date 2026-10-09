module MetaTest exposing (suite)

import Expect
import Json.Decode as D
import Meta
import Test exposing (Test, describe, test)


{-| Two pieces' meta exactly as the wasm module's `metas_json()` gives it
(the fire's palette cut to two colours).
-}
flags : String
flags =
    """{"pieces": [
  {"name":"braille wave","slug":"braille-wave","category":"generative","note":"two sine waves plotted in braille dots, eight to a cell","cols":40,"rows":10,"fps":30,"charset":"extended","options":"{\\"speed\\": 1.0}","clock":false,"palette":null,"ground":null,"cell":2},
  {"name":"quadrant fire","slug":"quadrant-fire","category":"effects","note":"the psx fire spread in quadrant blocks, shaded in the doom palette","cols":60,"rows":18,"fps":24,"charset":"basic","options":null,"clock":false,"palette":["#1f0707","#2f0f07"],"ground":"#070707","cell":2}
]}"""


{-| `Category::ALL` in crates/animate-unicode/src/meta.rs.
-}
rustCategories : List String
rustCategories =
    [ "scenes", "ui", "data", "type", "logos", "shapes", "space", "physics", "nature", "creatures", "objects", "generative", "effects" ]


suite : Test
suite =
    describe "Meta"
        [ test "decodes the wasm module's JSON, every field" <|
            \_ ->
                D.decodeString Meta.piecesDecoder flags
                    |> Result.map (List.map (\m -> ( m.slug, ( m.options, m.palette, m.ground ) )))
                    |> Expect.equal
                        (Ok
                            [ ( "braille-wave", ( Just "{\"speed\": 1.0}", Nothing, Nothing ) )
                            , ( "quadrant-fire", ( Nothing, Just [ "#1f0707", "#2f0f07" ], Just "#070707" ) )
                            ]
                        )
        , test "keeps the numbers and flags" <|
            \_ ->
                D.decodeString Meta.piecesDecoder flags
                    |> Result.map (List.map (\m -> [ m.cols, m.rows, m.fps, m.cell ]))
                    |> Expect.equal (Ok [ [ 40, 10, 30, 2 ], [ 60, 18, 24, 2 ] ])
        , test "a missing field is an error, not an empty list" <|
            \_ ->
                D.decodeString Meta.piecesDecoder """{"pieces": [{"name": "x"}]}"""
                    |> Result.toMaybe
                    |> Expect.equal Nothing
        , test "every category is in exactly one group" <|
            \_ ->
                List.concatMap .categories Meta.groups
                    |> List.sort
                    |> Expect.equal (List.sort rustCategories)
        ]
