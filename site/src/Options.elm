module Options exposing (Default(..), Edits, defaults, isValid, overrides)

{-| A piece's options on its page: the defaults from its meta, what the reader
has typed over them, and the JSON object of changes the `options` attribute
gets.
-}

import Dict exposing (Dict)
import Json.Decode as D
import Json.Encode as E


{-| An option's default, which decides its control.
-}
type Default
    = Number Float
    | Flag Bool
    | Text String
    | Other String -- any other JSON (a list, an object), shown as text and not editable


{-| What the reader typed, by option name. Numbers are kept as typed, so a
half-written "1." or "-" can sit in the field without being thrown away.
-}
type alias Edits =
    Dict String String


{-| The defaults, in the order the piece declares them, from `Meta.options`.
-}
defaults : String -> Result String (List ( String, Default ))
defaults json =
    D.decodeString (keyValuesInOrder defaultDecoder) json
        |> Result.mapError D.errorToString


{-| `D.keyValuePairs` makes no promise about order; the piece's own order is
the one to show, so read the keys first and then each value by key.
-}
keyValuesInOrder : D.Decoder a -> D.Decoder (List ( String, a ))
keyValuesInOrder value =
    D.keyValuePairs D.value
        |> D.andThen
            (\pairs ->
                pairs
                    |> List.map
                        (\( k, v ) ->
                            case D.decodeValue value v of
                                Ok a ->
                                    D.succeed ( k, a )

                                Err e ->
                                    D.fail (D.errorToString e)
                        )
                    |> List.foldr (D.map2 (::)) (D.succeed [])
            )


defaultDecoder : D.Decoder Default
defaultDecoder =
    D.oneOf
        [ D.map Number D.float
        , D.map Flag D.bool
        , D.map Text D.string
        , D.map (Other << E.encode 0) D.value
        ]


{-| Whether what was typed is a usable value for the option.
-}
isValid : Default -> String -> Bool
isValid default typed =
    case default of
        Number _ ->
            String.toFloat (String.trim typed) /= Nothing

        _ ->
            True


{-| The edits that are valid and differ from their default, as a JSON object
for the `options` attribute, in the defaults' order. Nothing when there are
none, so the tag and its snippet stay plain.
-}
overrides : List ( String, Default ) -> Edits -> Maybe String
overrides fields edits =
    let
        changed ( name, default ) =
            Dict.get name edits |> Maybe.andThen (encodeIfChanged default) |> Maybe.map (Tuple.pair name)
    in
    case List.filterMap changed fields of
        [] ->
            Nothing

        pairs ->
            Just (E.encode 0 (E.object pairs))


encodeIfChanged : Default -> String -> Maybe E.Value
encodeIfChanged default typed =
    case default of
        Number d ->
            String.toFloat (String.trim typed)
                |> Maybe.andThen
                    (\n ->
                        if n == d then
                            Nothing

                        else
                            Just (E.float n)
                    )

        Flag d ->
            if typed == "true" && not d then
                Just (E.bool True)

            else if typed == "false" && d then
                Just (E.bool False)

            else
                Nothing

        Text d ->
            if typed == d then
                Nothing

            else
                Just (E.string typed)

        Other _ ->
            Nothing
