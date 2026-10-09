module RouteTest exposing (suite)

import Expect
import Route exposing (Route(..))
import Test exposing (Test, describe, test)
import Url


at : String -> Maybe Route
at path =
    Url.fromString ("https://example.com" ++ path) |> Maybe.map Route.fromUrl


tidied : String -> Maybe (Maybe String)
tidied path =
    Url.fromString ("https://example.com" ++ path) |> Maybe.map Route.canonical


suite : Test
suite =
    describe "Route"
        [ test "the root is the index" <|
            \_ -> at "/" |> Expect.equal (Just Index)
        , test "one segment is a piece" <|
            \_ -> at "/donut" |> Expect.equal (Just (Piece "donut"))
        , test "a trailing slash is the same piece" <|
            \_ -> at "/donut/" |> Expect.equal (Just (Piece "donut"))
        , test "an unknown slug is still a piece route; the page says not found" <|
            \_ -> at "/nope" |> Expect.equal (Just (Piece "nope"))
        , test "deeper paths are not found" <|
            \_ -> at "/donut/extra" |> Expect.equal (Just NotFound)
        , test "a query or fragment does not change the route" <|
            \_ -> at "/braille-wave?x=1#top" |> Expect.equal (Just (Piece "braille-wave"))
        , test "tidy paths need no change" <|
            \_ -> List.map tidied [ "/", "/donut" ] |> Expect.equal [ Just Nothing, Just Nothing ]
        , test "a trailing or doubled slash is tidied" <|
            \_ -> List.map tidied [ "/donut/", "//donut" ] |> Expect.equal [ Just (Just "/donut"), Just (Just "/donut") ]
        ]
