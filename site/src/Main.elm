port module Main exposing (main)

{-| The site: every piece, grouped as in ascii.rest, with one page per piece.

The pieces come in as flags (their `Meta` as JSON, read from the wasm module
by main.ts before Elm starts) and are drawn by the `<unicode-art>` custom
element, which Elm treats as any other tag. Elm never touches a canvas or
wasm memory itself: the shell in web/ does that. Changing a tag's attributes
(options, mono) makes the element restart its piece.

-}

import Browser
import Browser.Dom as Dom
import Browser.Navigation as Nav
import Dict
import Html exposing (Html, a, button, code, div, h1, h2, h3, header, input, label, li, main_, nav, node, p, pre, section, span, text, ul)
import Html.Attributes as A exposing (attribute, class, classList, href)
import Html.Events exposing (onCheck, onClick, onInput)
import Html.Keyed as Keyed
import Json.Decode as D
import Meta exposing (Meta)
import Options
import Process
import Route exposing (Route)
import Task
import Url exposing (Url)


{-| Text for the clipboard; main.ts hands it to `navigator.clipboard`.
-}
port copy : String -> Cmd msg



-- MODEL


type alias Model =
    { key : Nav.Key
    , pieces : Result String (List Meta)
    , route : Route
    , edits : Options.Edits
    , mono : Bool
    , paper : Bool
    , copied : Maybe String
    }


init : D.Value -> Url -> Nav.Key -> ( Model, Cmd Msg )
init flags url key =
    ( { key = key
      , pieces = D.decodeValue Meta.piecesDecoder flags |> Result.mapError D.errorToString |> Result.map inOrder
      , route = Route.fromUrl url
      , edits = Dict.empty
      , mono = False
      , paper = False
      , copied = Nothing
      }
    , tidy key url
    )


{-| Sidebar order: by group and category, then by name.
-}
inOrder : List Meta -> List Meta
inOrder pieces =
    let
        order =
            List.concatMap .categories Meta.groups

        rank m =
            List.length (takeWhileNot m.category order)
    in
    List.sortBy (\m -> ( rank m, m.name )) pieces


takeWhileNot : String -> List String -> List String
takeWhileNot stop list =
    case list of
        [] ->
            []

        x :: rest ->
            if x == stop then
                []

            else
                x :: takeWhileNot stop rest


{-| Swaps a URL with a trailing slash for the tidy one, in place.
-}
tidy : Nav.Key -> Url -> Cmd Msg
tidy key url =
    case Route.canonical url of
        Just path ->
            Nav.replaceUrl key (path ++ (url.query |> Maybe.map ((++) "?") |> Maybe.withDefault "") ++ (url.fragment |> Maybe.map ((++) "#") |> Maybe.withDefault ""))

        Nothing ->
            Cmd.none



-- UPDATE


type Msg
    = LinkClicked Browser.UrlRequest
    | UrlChanged Url
    | Edit String String
    | ResetOptions
    | SetMono Bool
    | SetPaper Bool
    | Copy String String
    | ClearCopied String
    | NoOp


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LinkClicked (Browser.Internal url) ->
            ( model, Nav.pushUrl model.key (Url.toString url) )

        LinkClicked (Browser.External link) ->
            ( model, Nav.load link )

        UrlChanged url ->
            let
                route =
                    Route.fromUrl url

                fresh =
                    if route == model.route then
                        model

                    else
                        { model | route = route, edits = Dict.empty, mono = False, paper = False }
            in
            ( fresh, Cmd.batch [ tidy model.key url, scrollTo url.fragment ] )

        Edit name value ->
            ( { model | edits = Dict.insert name value model.edits }, Cmd.none )

        ResetOptions ->
            ( { model | edits = Dict.empty }, Cmd.none )

        SetMono on ->
            ( { model | mono = on }, Cmd.none )

        SetPaper on ->
            ( { model | paper = on }, Cmd.none )

        Copy id content ->
            ( { model | copied = Just id }, Cmd.batch [ copy content, Process.sleep 1500 |> Task.perform (\_ -> ClearCopied id) ] )

        ClearCopied id ->
            if model.copied == Just id then
                ( { model | copied = Nothing }, Cmd.none )

            else
                ( model, Cmd.none )

        NoOp ->
            ( model, Cmd.none )


{-| After a page change: to the fragment's element, or the top.
-}
scrollTo : Maybe String -> Cmd Msg
scrollTo fragment =
    case fragment of
        Just id ->
            Dom.getElement id
                |> Task.andThen (\el -> Dom.setViewport 0 el.element.y)
                |> Task.attempt (\_ -> NoOp)

        Nothing ->
            Task.perform (\_ -> NoOp) (Dom.setViewport 0 0)



-- VIEW


view : Model -> Browser.Document Msg
view model =
    case model.pieces of
        Err error ->
            { title = "error · animate-unicode"
            , body =
                [ main_ [ class "broken" ]
                    [ h1 [] [ text "the piece list did not decode" ]
                    , p [] [ text "main.ts passes every piece's Meta from the wasm module; Meta.elm's decoder rejected it:" ]
                    , pre [] [ text error ]
                    ]
                ]
            }

        Ok pieces ->
            let
                ( title, content ) =
                    page model pieces
            in
            { title = title
            , body = [ div [ class "layout" ] [ sidebar model.route pieces, main_ [ class "main" ] content ] ]
            }


page : Model -> List Meta -> ( String, List (Html Msg) )
page model pieces =
    case model.route of
        Route.Index ->
            ( "animate-unicode: animated unicode art for web pages", index pieces )

        Route.Piece slug ->
            case find slug pieces of
                Just m ->
                    ( m.name ++ " · animate-unicode", piecePage model pieces m )

                Nothing ->
                    ( "not found · animate-unicode", notFound (Just slug) )

        Route.NotFound ->
            ( "not found · animate-unicode", notFound Nothing )


find : String -> List Meta -> Maybe Meta
find slug pieces =
    List.head (List.filter (\m -> m.slug == slug) pieces)


inCategory : String -> List Meta -> List Meta
inCategory category =
    List.filter (\m -> m.category == category)


sidebar : Route -> List Meta -> Html Msg
sidebar route pieces =
    let
        current slug =
            if route == Route.Piece slug then
                [ attribute "aria-current" "page" ]

            else
                []

        category name =
            case inCategory name pieces of
                [] ->
                    []

                members ->
                    [ h3 [] [ text name ]
                    , ul [] (List.map (\m -> li [] [ a (href ("/" ++ m.slug) :: current m.slug) [ text m.name ] ]) members)
                    ]

        group g =
            case List.concatMap category g.categories of
                [] ->
                    []

                items ->
                    [ section [] (h2 [] [ text g.label ] :: items) ]
    in
    nav [ class "sidebar", attribute "aria-label" "Pieces" ]
        (p [ class "site" ] [ a [ href "/" ] [ text "animate-unicode" ] ] :: List.concatMap group Meta.groups)


index : List Meta -> List (Html Msg)
index pieces =
    let
        category name =
            case inCategory name pieces of
                [] ->
                    []

                members ->
                    [ section [ A.id name, class "category" ]
                        [ div [ class "head" ] [ h2 [] [ text name ], span [ class "count" ] [ text (String.fromInt (List.length members)) ] ]
                        , div [ class "grid" ] (List.map card members)
                        ]
                    ]

        card m =
            a [ class "card", href ("/" ++ m.slug) ]
                [ well m { big = False, mono = False, paper = False, options = Nothing }
                , span [ class "name" ] [ text m.name ]
                , span [ class "note" ] [ text m.note ]
                ]
    in
    header [ class "intro" ]
        [ h1 [] [ text "animated unicode art for web pages" ]
        , p [ class "lede" ]
            [ text
                (String.fromInt (List.length pieces)
                    ++ " pieces drawn in braille, blocks, box drawing and other unicode, written in Rust and compiled to WebAssembly. Each is one HTML tag."
                )
            ]
        ]
        :: List.concatMap category (List.concatMap .categories Meta.groups)


type alias Look =
    { big : Bool
    , mono : Bool
    , paper : Bool
    , options : Maybe String
    }


{-| A piece in its well. Keyed by slug, so moving to another piece replaces
the element (and frees its player) rather than re-pointing it.
-}
well : Meta -> Look -> Html Msg
well m look =
    let
        scene =
            m.ground /= Nothing && not look.mono

        ground =
            case ( m.ground, scene ) of
                ( Just g, True ) ->
                    [ A.style "background" g ]

                _ ->
                    []

        attrs =
            attribute "piece" m.slug
                :: (if look.mono then
                        [ attribute "mono" "" ]

                    else
                        []
                   )
                ++ (case look.options of
                        Just json ->
                            [ attribute "options" json ]

                        Nothing ->
                            []
                   )
    in
    div (classList [ ( "well", True ), ( "big", look.big ), ( "scene", scene ), ( "paper", look.paper ) ] :: ground)
        [ Keyed.node "div" [ class "art" ] [ ( m.slug, node "unicode-art" attrs [] ) ] ]


piecePage : Model -> List Meta -> Meta -> List (Html Msg)
piecePage model pieces m =
    let
        fields =
            Maybe.map Options.defaults m.options

        overrides =
            case fields of
                Just (Ok fs) ->
                    Options.overrides fs model.edits

                _ ->
                    Nothing
    in
    [ header []
        [ h1 [] [ text m.name ]
        , p [ class "lede" ] [ text m.note ]
        , p [ class "facts" ] (facts m)
        ]
    , div [ class "toggles" ] (toggles model m)
    , well m { big = True, mono = model.mono, paper = model.paper, options = overrides }
    ]
        ++ optionsBlock model fields
        ++ [ usage model m overrides, pager pieces m ]


facts : Meta -> List (Html Msg)
facts m =
    List.filterMap identity
        [ Just (a [ href ("/#" ++ m.category) ] [ text m.category ])
        , Just (span [] [ text (String.fromInt m.cols ++ "×" ++ String.fromInt m.rows) ])
        , Just
            (span []
                [ text
                    (if m.fps == 0 then
                        "still"

                     else
                        String.fromInt m.fps ++ " fps"
                    )
                ]
            )
        , Just (span [] [ text (m.charset ++ " glyphs") ])
        , m.palette
            |> Maybe.map
                (\colours ->
                    span []
                        [ text
                            (String.fromInt (List.length colours)
                                ++ " colours"
                                ++ (if m.ground /= Nothing then
                                        ", on a canvas"

                                    else
                                        ""
                                   )
                            )
                        ]
                )
        , if m.clock then
            Just (span [] [ text "reads the clock" ])

          else
            Nothing
        ]


toggles : Model -> Meta -> List (Html Msg)
toggles model m =
    List.filterMap identity
        [ if m.palette /= Nothing then
            Just (checkbox "mono" "one ink, as text" model.mono SetMono)

          else
            Nothing
        , if m.ground == Nothing || model.mono then
            Just (checkbox "paper" "dark ink on a light ground" model.paper SetPaper)

          else
            Nothing
        ]


checkbox : String -> String -> Bool -> (Bool -> Msg) -> Html Msg
checkbox name hint on toMsg =
    label [ class "toggle", A.title hint ] [ input [ A.type_ "checkbox", A.checked on, onCheck toMsg ] [], text name ]


optionsBlock : Model -> Maybe (Result String (List ( String, Options.Default ))) -> List (Html Msg)
optionsBlock model fields =
    case fields of
        Nothing ->
            []

        Just (Err error) ->
            [ section [ class "block" ] [ div [ class "head" ] [ h2 [] [ text "options" ] ], pre [ class "error" ] [ text error ] ] ]

        Just (Ok fs) ->
            [ section [ class "block options" ]
                [ div [ class "head" ]
                    [ h2 [] [ text "options" ]
                    , if Dict.isEmpty model.edits then
                        text ""

                      else
                        button [ class "button", A.type_ "button", onClick ResetOptions ] [ text "[reset]" ]
                    ]
                , div [ class "fields" ] (List.map (optionField model.edits) fs)
                ]
            ]


optionField : Options.Edits -> ( String, Options.Default ) -> Html Msg
optionField edits ( name, default ) =
    let
        typed shown =
            Dict.get name edits |> Maybe.withDefault shown
    in
    case default of
        Options.Number d ->
            let
                value =
                    typed (String.fromFloat d)
            in
            label [ class "field" ]
                [ span [] [ text name ]
                , input
                    [ A.type_ "number"
                    , A.step
                        (if d == toFloat (round d) then
                            "1"

                         else
                            "any"
                        )
                    , A.value value
                    , onInput (Edit name)
                    , attribute "aria-invalid"
                        (if Options.isValid default value then
                            "false"

                         else
                            "true"
                        )
                    ]
                    []
                ]

        Options.Flag d ->
            let
                on =
                    typed
                        (if d then
                            "true"

                         else
                            "false"
                        )
                        == "true"
            in
            label [ class "field" ]
                [ input
                    [ A.type_ "checkbox"
                    , A.checked on
                    , onCheck
                        (\b ->
                            Edit name
                                (if b then
                                    "true"

                                 else
                                    "false"
                                )
                        )
                    ]
                    []
                , span [] [ text name ]
                ]

        Options.Text d ->
            label [ class "field" ] [ span [] [ text name ], input [ A.type_ "text", A.value (typed d), onInput (Edit name) ] [] ]

        Options.Other json ->
            div [ class "field" ] [ span [] [ text name ], code [] [ text json ] ]


usage : Model -> Meta -> Maybe String -> Html Msg
usage model m overrides =
    let
        tag =
            "<unicode-art piece=\""
                ++ m.slug
                ++ "\""
                ++ (if model.mono then
                        " mono"

                    else
                        ""
                   )
                ++ (case overrides of
                        Just json ->
                            " options='" ++ String.replace "'" "&#39;" json ++ "'"

                        Nothing ->
                            ""
                   )
                ++ "></unicode-art>"

        snippet id caption content =
            div [ class "snippet" ]
                [ div [ class "head" ]
                    [ h3 [] [ text caption ]
                    , button [ class "button", A.type_ "button", onClick (Copy id content) ]
                        [ text
                            (if model.copied == Just id then
                                "[copied]"

                             else
                                "[copy]"
                            )
                        ]
                    ]
                , pre [] [ text content ]
                ]
    in
    section [ class "block" ]
        [ div [ class "head" ] [ h2 [] [ text "use it" ] ]
        , snippet "install" "install" "npm install animate-unicode"
        , snippet "import" "once, in your page's script" "import \"animate-unicode/element\";"
        , snippet "tag" "where it goes" tag
        ]


pager : List Meta -> Meta -> Html Msg
pager pieces m =
    let
        before =
            List.reverse (takeUntil m pieces)

        after =
            List.drop 1 (dropUntil m pieces)

        link label target =
            case target of
                Just t ->
                    a [ href ("/" ++ t.slug) ] [ text label ]

                Nothing ->
                    span [] []
    in
    nav [ class "pager", attribute "aria-label" "Neighbouring pieces" ]
        [ link ("< " ++ (List.head before |> Maybe.map .name |> Maybe.withDefault "")) (List.head before)
        , link ((List.head after |> Maybe.map .name |> Maybe.withDefault "") ++ " >") (List.head after)
        ]


takeUntil : Meta -> List Meta -> List Meta
takeUntil m list =
    case list of
        [] ->
            []

        x :: rest ->
            if x.slug == m.slug then
                []

            else
                x :: takeUntil m rest


dropUntil : Meta -> List Meta -> List Meta
dropUntil m list =
    case list of
        [] ->
            []

        x :: rest ->
            if x.slug == m.slug then
                list

            else
                dropUntil m rest


notFound : Maybe String -> List (Html Msg)
notFound slug =
    [ header []
        [ h1 [] [ text "not found" ]
        , p [ class "lede" ]
            [ text
                (case slug of
                    Just s ->
                        "There is no piece called \"" ++ s ++ "\"."

                    Nothing ->
                        "Nothing lives at this address."
                )
            ]
        , p [] [ a [ href "/" ] [ text "every piece" ] ]
        ]
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
