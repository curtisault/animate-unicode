module Main exposing (main)

{-| The site: every piece, by category, with one page per piece.

The pieces come in as flags (their `Meta` as JSON, read from the wasm module
by main.ts before Elm starts) and are drawn by the `<unicode-art>` custom
element, which Elm treats as any other tag. Elm never touches a canvas or
wasm memory itself: the shell in web/ does that.

-}

import Browser
import Browser.Navigation as Nav
import Html exposing (Html, a, div, h1, h2, li, node, p, pre, text, ul)
import Html.Attributes exposing (attribute, class, href)
import Json.Decode as D
import Url exposing (Url)



-- MODEL


type alias Meta =
    { name : String
    , slug : String
    , category : String
    , note : String
    , cols : Int
    , rows : Int
    , fps : Int
    , charset : String
    , palette : Maybe (List String)
    }


type alias Model =
    { key : Nav.Key
    , pieces : List Meta
    , route : Route
    }


type Route
    = Index
    | Piece String
    | NotFound


metaDecoder : D.Decoder Meta
metaDecoder =
    D.map8 Meta
        (D.field "name" D.string)
        (D.field "slug" D.string)
        (D.field "category" D.string)
        (D.field "note" D.string)
        (D.field "cols" D.int)
        (D.field "rows" D.int)
        (D.field "fps" D.int)
        (D.field "charset" D.string)
        |> D.andThen (\partial -> D.map partial (D.field "palette" (D.nullable (D.list D.string))))


{-| Sidebar order. Mirrors `Category::ALL` in crates/animate-unicode/src/meta.rs.
-}
categories : List String
categories =
    [ "scenes", "ui", "data", "type", "logos", "shapes", "space", "physics", "nature", "creatures", "objects", "generative", "effects" ]


route : Url -> Route
route url =
    case String.split "/" (String.dropLeft 1 url.path) |> List.filter ((/=) "") of
        [] ->
            Index

        [ slug ] ->
            Piece slug

        _ ->
            NotFound


init : D.Value -> Url -> Nav.Key -> ( Model, Cmd Msg )
init flags url key =
    let
        pieces =
            D.decodeValue (D.field "pieces" (D.list metaDecoder)) flags
                |> Result.withDefault []
    in
    ( { key = key, pieces = pieces, route = route url }, Cmd.none )



-- UPDATE


type Msg
    = LinkClicked Browser.UrlRequest
    | UrlChanged Url


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LinkClicked (Browser.Internal url) ->
            ( model, Nav.pushUrl model.key (Url.toString url) )

        LinkClicked (Browser.External href) ->
            ( model, Nav.load href )

        UrlChanged url ->
            ( { model | route = route url }, Cmd.none )



-- VIEW


view : Model -> Browser.Document Msg
view model =
    { title =
        case model.route of
            Piece slug ->
                slug ++ " · animate-unicode"

            _ ->
                "animate-unicode"
    , body =
        [ div [ class "layout" ]
            [ sidebar model
            , div [ class "main" ] (main_ model)
            ]
        ]
    }


sidebar : Model -> Html Msg
sidebar model =
    let
        current slug =
            case model.route of
                Piece s ->
                    if s == slug then
                        [ attribute "aria-current" "page" ]

                    else
                        []

                _ ->
                    []

        group category =
            case List.filter (\m -> m.category == category) model.pieces of
                [] ->
                    []

                members ->
                    [ h2 [] [ text category ]
                    , ul [] (List.map (\m -> li [] [ a (href ("/" ++ m.slug) :: current m.slug) [ text m.name ] ]) members)
                    ]
    in
    div [ class "sidebar" ]
        (h1 [] [ a [ href "/" ] [ text "animate-unicode" ] ]
            :: List.concatMap group categories
        )


main_ : Model -> List (Html Msg)
main_ model =
    case model.route of
        Index ->
            List.map pieceView model.pieces

        Piece slug ->
            case List.filter (\m -> m.slug == slug) model.pieces of
                m :: _ ->
                    [ pieceView m ]

                [] ->
                    notFound

        NotFound ->
            notFound


notFound : List (Html Msg)
notFound =
    [ h2 [] [ text "no such piece" ], p [] [ a [ href "/" ] [ text "every piece" ] ] ]


pieceView : Meta -> Html Msg
pieceView m =
    div [ class "piece" ]
        [ h2 [] [ a [ href ("/" ++ m.slug) ] [ text m.name ] ]
        , p [] [ text (m.note ++ " · " ++ String.fromInt m.cols ++ "×" ++ String.fromInt m.rows ++ " · " ++ String.fromInt m.fps ++ " fps · " ++ m.charset) ]
        , node "unicode-art" [ attribute "piece" m.slug ] []
        , pre [ class "snippet" ] [ text ("<unicode-art piece=\"" ++ m.slug ++ "\"></unicode-art>") ]
        ]



-- MAIN


main : Program D.Value Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = \_ -> Sub.none
        , onUrlRequest = LinkClicked
        , onUrlChange = UrlChanged
        }
