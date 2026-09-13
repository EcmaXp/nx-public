begin
    set --local unique_path
    for entry in $PATH
        contains -- "$entry" $unique_path; or set --append unique_path "$entry"
    end
    set --global --export PATH $unique_path
end
