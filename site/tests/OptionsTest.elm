module OptionsTest exposing (suite)

import Dict
import Expect
import Options exposing (Default(..))
import Test exposing (Test, describe, test)


fields : List ( String, Default )
fields =
    case Options.defaults """{"speed": 1.0, "count": 3, "glow": true, "label": "hi", "stops": [1, 2]}""" of
        Ok fs ->
            fs

        Err _ ->
            []


suite : Test
suite =
    describe "Options"
        [ test "defaults keep the piece's order and pick a control by type" <|
            \_ ->
                fields
                    |> Expect.equal
                        [ ( "speed", Number 1 ), ( "count", Number 3 ), ( "glow", Flag True ), ( "label", Text "hi" ), ( "stops", Other "[1,2]" ) ]
        , test "no edits, no options attribute" <|
            \_ -> Options.overrides fields Dict.empty |> Expect.equal Nothing
        , test "only changed, valid values are passed, in the piece's order" <|
            \_ ->
                Options.overrides fields (Dict.fromList [ ( "label", "yo" ), ( "speed", "2.5" ), ( "count", "3" ), ( "glow", "false" ) ])
                    |> Expect.equal (Just """{"speed":2.5,"glow":false,"label":"yo"}""")
        , test "a half-typed number is kept out until it parses" <|
            \_ ->
                Options.overrides fields (Dict.fromList [ ( "speed", "-" ) ])
                    |> Expect.equal Nothing
        , test "isValid flags what cannot be a number" <|
            \_ -> [ Options.isValid (Number 1) "2", Options.isValid (Number 1) "abc", Options.isValid (Text "") "abc" ] |> Expect.equal [ True, False, True ]
        , test "bad JSON is an error" <|
            \_ -> Options.defaults "{" |> Result.toMaybe |> Expect.equal Nothing
        ]
