module Route exposing (Route(..), canonical, fromUrl)

{-| The site's URLs: `/` and `/<slug>`. Any other path is not found; whether a
slug names a piece is for the page to decide.
-}

import Url exposing (Url)


type Route
    = Index
    | Piece String
    | NotFound


fromUrl : Url -> Route
fromUrl url =
    case segments url of
        [] ->
            Index

        [ slug ] ->
            Piece slug

        _ ->
            NotFound


{-| The path to show instead, when the URL has a trailing slash (or doubled
slashes): `/donut/` becomes `/donut`.
-}
canonical : Url -> Maybe String
canonical url =
    let
        tidy =
            "/" ++ String.join "/" (segments url)
    in
    if tidy == url.path then
        Nothing

    else
        Just tidy


segments : Url -> List String
segments url =
    String.split "/" url.path |> List.filter ((/=) "")
